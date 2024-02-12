use image::DynamicImage;
use std::process::Command;
use tempfile::Builder;

pub fn file_to_img(file_name: &str) -> DynamicImage {
    if file_name.ends_with(".pdf") {
        pdf_to_img(file_name)
    } else {
        image::open(file_name).expect("Failed to open file")
    }
}

fn pdf_to_img(file_name: &str) -> DynamicImage {
    let temp_file = Builder::new()
        .suffix(".png")
        .tempfile()
        .expect("Failed to create temp file");

    let path_without = temp_file.path().with_extension("");
    let temp_file_without_extension = path_without.to_str().unwrap();

    let status = Command::new("pdftoppm")
        .args([
            "-png",
            "-singlefile",
            file_name,
            temp_file_without_extension,
        ])
        .status()
        .expect("failed to execute process");

    assert!(status.success(), "pdftoppm command failed");

    image::io::Reader::open(temp_file.path())
        .expect("Failed to open temp file")
        .decode()
        .expect("Failed to decode image")
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
