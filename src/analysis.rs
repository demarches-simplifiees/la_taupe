use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    datamatrix::fetch_datamatrix,
    file_utils::{bytes_to_img, pdf_bytes_to_string},
    twoddoc::{ddoc::Ddoc, parse},
};

use crate::rib::Rib;

#[derive(Deserialize, Serialize)]
pub struct Analysis {
    #[serde(rename = "2ddoc")]
    pub ddoc: Option<Ddoc>,
    pub rib: Option<Rib>,
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

fn vec_to_rib(content: Vec<u8>) -> Result<Rib, String> {
    let string_rib = pdf_bytes_to_string(content)?;
    Rib::try_from(string_rib)
}

fn vec_to_ddoc(content: Vec<u8>) -> Result<Ddoc, String> {
    let img = bytes_to_img(content)?;
    let datamatrix = fetch_datamatrix(img)
        .ok_or_else(|| "Failed to fetch Data Matrix from image".to_string())?;

    parse(&datamatrix).ok_or_else(|| "Failed to parse 2DDoc".to_string())
}

impl TryFrom<(Vec<u8>, Option<Hint>)> for Analysis {
    type Error = String;

    fn try_from((content, hint): (Vec<u8>, Option<Hint>)) -> Result<Self, String> {
        match hint {
            Some(Hint::Type(Type::Rib)) => {
                let rib = vec_to_rib(content)?;

                Ok(Analysis {
                    rib: Some(rib),
                    ddoc: None,
                })
            }
            Some(Hint::Type(Type::Twoddoc)) => {
                let ddoc = vec_to_ddoc(content)?;

                Ok(Analysis {
                    ddoc: Some(ddoc),
                    rib: None,
                })
            }
            None => {
                let rib = vec_to_rib(content.clone());

                if rib.is_ok() {
                    let rib = rib.unwrap();
                    return Ok(Analysis {
                        rib: Some(rib),
                        ddoc: None,
                    });
                }

                let ddoc = vec_to_ddoc(content);

                if ddoc.is_ok() {
                    let ddoc = ddoc.unwrap();
                    return Ok(Analysis {
                        ddoc: Some(ddoc),
                        rib: None,
                    });
                }

                Err("Failed to parse content as either RIB or 2DDoc".to_string())
            }
        }
    }
}

impl TryFrom<&Path> for Analysis {
    type Error = String;

    fn try_from(file_path: &Path) -> Result<Self, String> {
        let content = std::fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;
        Analysis::try_from((content, None))
    }
}
