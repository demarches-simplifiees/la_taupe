use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::file_utils::pdf_to_img_bytes;
use crate::ocr::Ocr::{Ocrs, Tesseract};
use crate::rib::Rib;
use crate::{
    datamatrix::fetch_datamatrix,
    file_utils::{bytes_to_img, pdf_bytes_to_string},
    ocr::image_bytes_to_string,
    twoddoc::{ddoc::Ddoc, parse},
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "hint")]
pub enum Analysis {
    #[serde(rename = "2ddoc_and_rib")]
    DdocAndRib {
        #[serde(rename = "2ddoc")]
        ddoc: Option<Ddoc>,
        rib: Option<Rib>,
    },
    #[serde(rename = "rib")]
    Rib { rib: Option<Rib> },
    #[serde(rename = "2ddoc")]
    Ddoc {
        #[serde(rename = "2ddoc")]
        ddoc: Option<Ddoc>,
    },
}

#[derive(Deserialize, Copy, Clone, Debug)]
pub enum Hint {
    #[serde(rename = "type")]
    Type(Type),
}

#[derive(Deserialize, Copy, Clone, Debug)]
pub enum Type {
    #[serde(rename = "rib")]
    Rib,
    #[serde(rename = "2ddoc")]
    Twoddoc,
}

fn vec_to_rib(content: Vec<u8>) -> Result<Option<Rib>, String> {
    let filetype = tree_magic_mini::from_u8(&content);

    if filetype == "application/pdf" {
        let string_rib = pdf_bytes_to_string(content.clone());

        if !string_rib.trim().is_empty() {
            Ok(Rib::parse(string_rib))
        } else {
            let img = pdf_to_img_bytes(content);
            Ok(image_to_rib(img))
        }
    } else if filetype == "image/png" || filetype == "image/jpeg" {
        Ok(image_to_rib(content))
    } else if filetype == "text/plain" {
        let string_rib = String::from_utf8(content)
            .map_err(|_| "Failed to convert bytes to string".to_string())?;
        Ok(Rib::parse(string_rib))
    } else {
        Err(format!("Unsupported file type: {}", filetype))
    }
}

fn image_to_rib(content: Vec<u8>) -> Option<Rib> {
    let string_rib_tesseract = image_bytes_to_string(content.clone(), Tesseract);
    if let Some(rib) = Rib::parse(string_rib_tesseract) {
        return Some(rib);
    }

    let string_rib_ocrs = image_bytes_to_string(content, Ocrs);
    Rib::parse(string_rib_ocrs)
}

fn vec_to_ddoc(content: Vec<u8>) -> Result<Option<Ddoc>, String> {
    let img = bytes_to_img(content)?;

    if let Some(datamatrix) = fetch_datamatrix(img) {
        Ok(parse(&datamatrix))
    } else {
        Ok(None)
    }
}

impl TryFrom<(Vec<u8>, Option<Hint>)> for Analysis {
    type Error = String;

    fn try_from((content, hint): (Vec<u8>, Option<Hint>)) -> Result<Self, String> {
        match hint {
            Some(Hint::Type(Type::Rib)) => {
                let rib = vec_to_rib(content)?;

                Ok(Analysis::Rib { rib })
            }
            Some(Hint::Type(Type::Twoddoc)) => {
                let ddoc = vec_to_ddoc(content)?;

                Ok(Analysis::Ddoc { ddoc })
            }
            None => {
                let rib = vec_to_rib(content.clone()).unwrap_or(None);
                let ddoc = vec_to_ddoc(content).unwrap_or(None);

                Ok(Analysis::DdocAndRib { ddoc, rib })
            }
        }
    }
}

impl TryFrom<(&Path, Option<Hint>)> for Analysis {
    type Error = String;

    fn try_from((file_path, hint): (&Path, Option<Hint>)) -> Result<Self, String> {
        let content =
            std::fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;
        Analysis::try_from((content, hint))
    }
}
