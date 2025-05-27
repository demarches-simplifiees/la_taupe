use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    datamatrix::fetch_datamatrix,
    file_utils::{bytes_to_img, file_to_img},
    twoddoc::{ddoc::Ddoc, parse},
};

#[derive(Deserialize, Serialize)]
pub struct Analysis {
    // serialize as "2ddoc" instead of "ddoc
    #[serde(rename = "2ddoc")]
    pub ddoc: Option<Ddoc>,
}

impl TryFrom<Vec<u8>> for Analysis {
    type Error = String;

    fn try_from(content: Vec<u8>) -> Result<Self, String> {
        let img = bytes_to_img(content)?;
        let datamatrix = fetch_datamatrix(img);

        Ok(Analysis {
            ddoc: datamatrix.map(|datamatrix| parse(&datamatrix).unwrap()),
        })
    }
}

impl TryFrom<&Path> for Analysis {
    type Error = String;

    fn try_from(file_path: &Path) -> Result<Self, String> {
        let img = file_to_img(file_path)?;
        let datamatrix = fetch_datamatrix(img);

        Ok(Analysis {
            ddoc: datamatrix.map(|datamatrix| parse(&datamatrix).unwrap()),
        })
    }
}
