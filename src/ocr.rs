use std::{
    io::{Cursor, Write},
    process::{Command, Stdio},
};

use image::{DynamicImage, ImageFormat};
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;

const DETECTION_MODEL: &[u8] = include_bytes!("../models/text-detection.rten");
const RECOGNITION_MODEL: &[u8] = include_bytes!("../models/text-recognition.rten");

pub fn image_bytes_to_string(content: Vec<u8>) -> String {
    let img = image::load_from_memory(&content).expect("Failed to load image from bytes");

    image_to_string(img)
}

pub fn image_to_string(img: DynamicImage) -> String {
    img_to_string_using_tesseract(img)
}

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

pub fn increase_image_size_if_needed(img: DynamicImage) -> DynamicImage {
    // si la largeur ou la hauteur est inférieur a 500 on multiply par 2
    if img.width() >= 500 && img.height() >= 500 {
        return img;
    }

    // increase * 2 if the image is too small
    img.resize(
        img.width() * 2,
        img.height() * 2,
        image::imageops::FilterType::Lanczos3,
    )
}

pub fn img_to_string_using_tesseract(img: DynamicImage) -> String {
    let img = increase_image_size_if_needed(img);

    let mut buffer = Cursor::new(Vec::new());
    img.write_to(&mut buffer, ImageFormat::Png).unwrap();
    let vec = buffer.into_inner();

    let mut child = Command::new("tesseract")
        .args([
            "--psm",
            "12", // sparse text with orientation and script detection
            "--oem",
            "2", // Original Tesseract Engine (work with characters recognition, good for
            // IBAN Numbers) + LSTM (work with words recognition, good for text)
            "-c",
            "preserve_interword_spaces=1", // Try to keep the layout
            "-l",
            "fra",
            "-",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start pdftotext");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin.write_all(&vec).expect("Failed to write to stdin");
    });

    let output = child.wait_with_output().expect("Failed to wait on child");

    String::from_utf8_lossy(&output.stdout).to_string()
}
