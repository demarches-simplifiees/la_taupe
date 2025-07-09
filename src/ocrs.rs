use image::DynamicImage;
use ocrs::{ImageSource, OcrEngine, OcrEngineParams, TextItem, TextLine};
use regex::Regex;
use rten::Model;

use crate::shapes::{Anchor, Point};

const DETECTION_MODEL: &[u8] = include_bytes!("../models/text-detection.rten");
const RECOGNITION_MODEL: &[u8] = include_bytes!("../models/text-recognition.rten");

pub fn image_to_string_using_ocrs(img: DynamicImage) -> String {
    let img = img.into_rgb8();

    #[allow(clippy::const_is_empty)]
    if DETECTION_MODEL.is_empty() || RECOGNITION_MODEL.is_empty() {
        panic!("--> ocrs models are empty in models/ directory. Please run `download_models.sh` to download the models.");
    }

    // 10 ms to load the models
    let detection_model = Model::load_static_slice(DETECTION_MODEL).unwrap();
    let recognition_model = Model::load_static_slice(RECOGNITION_MODEL).unwrap();

    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })
    .unwrap();

    // Apply standard image pre-processing expected by this library (convert
    // to greyscale, map range to [-0.5, 0.5]).
    let img_source = ImageSource::from_bytes(img.as_raw(), img.dimensions()).unwrap();
    let ocr_input = engine.prepare_input(img_source).unwrap();

    // Get oriented bounding boxes of text words in input image.
    let word_rects = engine.detect_words(&ocr_input).unwrap();

    // Group words into lines. Each line is represented by a list of word
    // bounding boxes.
    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);

    // Recognize the characters in each line.
    let line_texts = engine.recognize_text(&ocr_input, &line_rects).unwrap();

    line_texts
        .iter()
        .flatten()
        // Filter likely spurious detections. With future model improvements
        // this should become unnecessary.
        .filter(|l| l.to_string().len() > 1)
        .map(|l| l.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn ocrs_anchors(
    img: &DynamicImage,
    word_regex: &Regex,
    line_regex: Option<&Regex>,
) -> (String, Vec<TextLine>, Vec<Anchor>) {
    let img = img.clone().into_rgb8();

    #[allow(clippy::const_is_empty)]
    if DETECTION_MODEL.is_empty() || RECOGNITION_MODEL.is_empty() {
        panic!("--> ocrs models are empty in models/ directory. Please run `download_models.sh` to download the models.");
    }

    // 10 ms to load the models
    let detection_model = Model::load_static_slice(DETECTION_MODEL).unwrap();
    let recognition_model = Model::load_static_slice(RECOGNITION_MODEL).unwrap();

    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })
    .unwrap();

    // Apply standard image pre-processing expected by this library (convert
    // to greyscale, map range to [-0.5, 0.5]).
    let img_source = ImageSource::from_bytes(img.as_raw(), img.dimensions()).unwrap();
    let ocr_input = engine.prepare_input(img_source).unwrap();

    // Get oriented bounding boxes of text words in input image.
    let word_rects = engine.detect_words(&ocr_input).unwrap();

    // Group words into lines. Each line is represented by a list of word
    // bounding boxes.
    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);

    // Recognize the characters in each line.
    let text_lines = engine
        .recognize_text(&ocr_input, &line_rects)
        .unwrap()
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<TextLine>>();

    let text = text_lines
        .iter()
        // Filter likely spurious detections. With future model improvements
        // this should become unnecessary.
        .filter(|l| l.to_string().len() > 1)
        .map(|l| l.to_string())
        .collect::<Vec<String>>()
        .join("\n");

    (
        text,
        text_lines.clone(),
        extract_anchors(text_lines, word_regex, line_regex),
    )
}

pub fn extract_anchors(
    text_lines: Vec<TextLine>,
    word_regex: &Regex,
    line_regex: Option<&Regex>,
) -> Vec<Anchor> {
    text_lines
        .iter()
        .filter(|line| {
            if line_regex.is_none() {
                return true;
            }
            line_regex.unwrap().is_match(&line.to_string())
        })
        .flat_map(|line| line.words())
        .filter(|word| word_regex.is_match(&word.to_string()))
        .map(|word| {
            let [p1, _, p3, _, ..] = word
                .rotated_rect()
                .corners()
                .map(|point| [point.x.round() as u32, point.y.round() as u32]);

            Anchor::new(Point::new(p3[0], p3[1]), Point::new(p1[0], p1[1]))
        })
        .collect::<Vec<Anchor>>()
}
