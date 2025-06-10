use image::DynamicImage;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

pub fn file_to_img(file_name: &Path) -> Result<DynamicImage, String> {
    let content = std::fs::read(file_name).map_err(|e| format!("Failed to read file: {}", e))?;
    bytes_to_img(content)
}

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
    let filetype = tree_magic_mini::from_u8(&bytes);

    if filetype != "application/pdf" {
        return Err("Only PDF files are supported for RIB analysis".to_string());
    };

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

    let lines = String::from_utf8_lossy(&output.stdout).to_string();

    if lines.lines().count() == 1 && lines.trim().is_empty() {
        Err("Failed to extract text from PDF".to_string())
    } else {
        Ok(lines)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_to_img() {
        let img_from_pdf = file_to_img(Path::new(
            "tests/fixtures/2ddoc/justificatif_de_domicile.pdf",
        ))
        .unwrap();
        assert_eq!(img_from_pdf.width(), 1241);
        assert_eq!(img_from_pdf.height(), 1754);

        let img_from_png = file_to_img(Path::new(
            "tests/fixtures/2ddoc/justificatif_de_domicile.png",
        ))
        .unwrap();
        assert_eq!(img_from_png.width(), 119);
        assert_eq!(img_from_png.height(), 122);
    }
}
