use image::DynamicImage;
use std::{
    io::Write,
    process::{Command, Stdio},
};

pub fn file_to_img(file_name: &str) -> DynamicImage {
    let content = std::fs::read(file_name).expect("Failed to read file");
    bytes_to_img(content)
}

pub fn bytes_to_img(bytes: Vec<u8>) -> DynamicImage {
    let filetype = tree_magic_mini::from_u8(&bytes);

    match filetype {
        "application/pdf" => pdf_to_img(bytes),
        _ => image::load_from_memory(&bytes).expect("Failed to load image from bytes")
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
        let img_from_pdf = file_to_img("tests/fixtures/2ddoc/justificatif_de_domicile.pdf");
        assert_eq!(img_from_pdf.width(), 1241);
        assert_eq!(img_from_pdf.height(), 1754);

        let img_from_png = file_to_img("tests/fixtures/2ddoc/justificatif_de_domicile.png");
        assert_eq!(img_from_png.width(), 119);
        assert_eq!(img_from_png.height(), 122);
    }
}
