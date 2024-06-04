use super::entete::Entete;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct Ddoc {
    pub entete: Entete,
    pub data: HashMap<String, String>,
}

impl Ddoc {
    pub fn new(entete: Entete, data: HashMap<String, String>) -> Self {
        Ddoc { entete, data }
    }
}
