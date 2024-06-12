use serde::{Deserialize, Serialize};

use crate::{
    datamatrix::fetch_datamatrix,
    file_utils::bytes_to_img,
    twoddoc::{ddoc::Ddoc, parse},
};

#[derive(Deserialize, Serialize)]
pub struct Analysis {
    pub ddoc: Option<Ddoc>,
}

impl Analysis {
    pub fn try_into(content: Vec<u8>) -> Result<Self, String> {
        let img = bytes_to_img(content)?;
        let datamatrix = fetch_datamatrix(img);

        Ok(Analysis {
            ddoc: datamatrix.map(|datamatrix| parse(&datamatrix).unwrap()),
        })
    }
}
