use super::utils::{union_of_legit_symbol, BoxedParser};
use serde::{Deserialize, Deserializer};

static STRUCTURE_JSON: &str = include_str!("structure.json");

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DataStructure {
    pub id: String,
    #[serde(deserialize_with = "from_string")]
    pub min: usize,
    #[serde(deserialize_with = "from_string")]
    pub max: usize,
    pub nature: String, // ["Alphanumérique", "Numérique", "Alphanumérique et symboles", "Alphabétique", "Float"]
    pub nom: String,
    pub description: String,
}

fn from_string<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    let r = match s {
        "Aucune" => usize::MAX,
        _ => s.parse().expect("fail"),
    };

    Ok(r)
}

pub fn data_structure_from_json() -> Vec<DataStructure> {
    serde_json::from_str(STRUCTURE_JSON).unwrap()
}

pub fn data_structure<'a>(id: &str) -> BoxedParser<'a> {
    let binding = data_structure_from_json();
    let data = binding.iter().find(|x| x.id == id);

    match data {
        Some(d) => union_of_legit_symbol(d.min, d.max),
        None => panic!("Unknown data id {}", id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structure() {
        let data = data_structure_from_json();
        assert_eq!(data[0].id, "01");
        assert_eq!(data[0].min, 0);
        assert_eq!(data[0].max, usize::MAX);
        assert_eq!(data[0].nature, "Alphanumérique");
        assert_eq!(data[0].nom, "Identifiant unique du document.");
        assert_eq!(&data[0].description[..22], "Cet identifiant permet");
    }
}
