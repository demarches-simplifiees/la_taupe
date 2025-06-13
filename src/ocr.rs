use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;

const DETECTION_MODEL: &[u8] = include_bytes!("../models/text-detection.rten");
const RECOGNITION_MODEL: &[u8] = include_bytes!("../models/text-recognition.rten");

pub fn image_bytes_to_string(content: Vec<u8>) -> String {
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

    // build image from Vec<u8>
    let img = image::load_from_memory(&content)
        .expect("Failed to load image from bytes")
        .into_rgb8();

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
