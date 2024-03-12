use super::utils::{alphanumeric, alphanumeric_space_slash, digit, digit_and_comma, BoxedParser};
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
    let data = binding.iter().find(|x| x.id == id).unwrap();

    match id {
        "01" => alphanumeric(data.min, data.max),
        "02" => alphanumeric(data.min, data.max),
        "10" => alphanumeric_space_slash(data.min, data.max),
        "18" => alphanumeric(data.min, data.max),
        "20" => alphanumeric_space_slash(data.min, data.max),
        "21" => alphanumeric_space_slash(data.min, data.max),
        "22" => alphanumeric_space_slash(data.min, data.max),
        "23" => alphanumeric_space_slash(data.min, data.max),
        "25" => alphanumeric_space_slash(data.min, data.max),
        "24" => alphanumeric(data.min, data.max),
        "26" => alphanumeric(data.min, data.max),
        "41" => digit_and_comma(data.min, data.max),
        "43" => digit_and_comma(data.min, data.max),
        "44" => alphanumeric(data.min, data.max),
        "45" => digit(data.min, data.max),
        "46" => alphanumeric_space_slash(data.min, data.max),
        "47" => digit(data.min, data.max),
        "48" => alphanumeric_space_slash(data.min, data.max),
        "49" => digit(data.min, data.max),
        "4A" => digit(data.min, data.max),
        _ => panic!("Unknwonw data id {}", id),
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
