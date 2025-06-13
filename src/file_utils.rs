use image::DynamicImage;
use std::{
    io::Write,
    process::{Command, Stdio},
};

use crate::ocr::image_to_string;

pub fn bytes_to_img(bytes: Vec<u8>) -> Result<DynamicImage, String> {
    let filetype = tree_magic_mini::from_u8(&bytes);

    match filetype {
        "application/pdf" => Ok(pdf_to_img(bytes)),
        "image/png" | "image/jpeg" => {
            Ok(image::load_from_memory(&bytes).expect("Failed to load image from bytes"))
        }
        _ => Err(format!("Unsupported file type: {}", filetype)),
    }
}

pub fn pdf_bytes_to_string(bytes: Vec<u8>) -> Result<String, String> {
    let cloned_bytes = bytes.clone();
    let mut child = Command::new("pdftotext")
        .args(["-layout", "-", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start pdftotext");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin.write_all(&bytes).expect("Failed to write to stdin");
    });

    let output = child.wait_with_output().expect("Failed to wait on child");

    let mut text = String::from_utf8_lossy(&output.stdout).to_string();

    if !text.trim().is_empty() {
        return Ok(text);
    }

    let img = pdf_to_img(cloned_bytes);
    text = image_to_string(img);

    if !text.trim().is_empty() {
        return Ok(text);
    }

    Err("Failed to extract text from PDF".to_string())
}

fn pdf_to_img(file: Vec<u8>) -> DynamicImage {
    let mut child = Command::new("pdftoppm")
        .args(["-png", "-singlefile"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin.write_all(&file).expect("Failed to write to stdin");
    });

    let output = child.wait_with_output().expect("Failed to wait on child");

    image::load_from_memory(&output.stdout).expect("Failed to load image from bytes")
}
