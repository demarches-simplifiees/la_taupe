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
    pub fn new(content: Vec<u8>) -> Self {
        let img = bytes_to_img(content);
        let datamatrix = fetch_datamatrix(img);

        Analysis {
            ddoc: datamatrix.map(|datamatrix| parse(&datamatrix).unwrap()),
        }
    }
}
