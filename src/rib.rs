use iban::Iban;
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::text_utils::clean;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Rib {
    titulaire: Option<Vec<String>>,
    iban: String,
    bic: Option<String>,
}

impl TryFrom<String> for Rib {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        let lines = clean(text.clone());

        let titulaire = extract_titulaire(lines.clone());
        let bic = extract_fr_bic(&text);

        if let Some(iban) = extract_iban(text) {
            Ok(Rib {
                titulaire,
                iban,
                bic,
            })
        } else {
            Err(format!("No IBAN found in the text. Lines: {:?}", lines))
        }
    }
}

fn extract_titulaire(lines: Vec<String>) -> Option<Vec<String>> {
    let titulaire = Regex::new(r"(?i)titulaire|intitulé du compte").unwrap();
    let code_postal = Regex::new(r"^\d{5}").unwrap();
    let domiciliation = Regex::new(r"(?i)domiciliation").unwrap();
    let identification = Regex::new(r"(?i)identification").unwrap();

    let titulaire_index = lines.iter().position(|x| titulaire.is_match(x));

    let code_postal_index = lines[titulaire_index?..]
        .iter()
        .position(|x| code_postal.is_match(x));

    let domiciliation_index = lines[titulaire_index?..]
        .iter()
        .position(|x| domiciliation.is_match(x))
        .map(|index| index - 1); // -1 because we exclude the domiciliation line itself

    let identification_index = lines[titulaire_index?..]
        .iter()
        .position(|x| identification.is_match(x))
        .map(|index| index - 1); // -1 because we exclude the identification line itself

    let end_index = [code_postal_index, domiciliation_index, identification_index]
        .iter()
        .filter_map(|&x| x)
        .min();

    match (titulaire_index, end_index) {
        (Some(titulaire_index), Some(code_postal_index)) => {
            let titulaire = lines[(titulaire_index + 1)..=(code_postal_index + titulaire_index)]
                .iter()
                // on enleve ": " en debut de ligne
                .map(|x| x.trim_start_matches(": ").to_string())
                .collect::<Vec<String>>();

            if titulaire.is_empty() {
                None
            } else {
                Some(titulaire)
            }
        }
        _ => None,
    }
}

fn extract_iban(text: String) -> Option<String> {
    let french_iban_re = Regex::new(r"(?<iban>FR[[:digit:]]{2}([[:space:]]*[[:alnum:]]{4}){5})([[:space:]]*[[:alnum:]][[:digit:]]{2})").unwrap();

    let mut iban = french_iban_re
        .find(&text)
        .map(|x| x.as_str().to_string())
        // change multiple spaces by one space
        .map(|x| x.split_whitespace().collect::<Vec<&str>>().join(" "));

    // sometimes the iban is written with weird space (credit_agricole_2.txt)
    // so we try to match by removing all spaces
    if iban.is_none() {
        let text_without_spaces = text.replace(" ", "");
        iban = french_iban_re
            .find(&text_without_spaces)
            .map(|x| x.as_str().to_string());
    }

    iban?.parse::<Iban>().ok().map(|x| x.to_string())
}

fn extract_fr_bic(content: &str) -> Option<String> {
    let fr_without_space = Regex::new(r"[A-Z]{4}FR[A-Z0-9]{2}([A-Z0-9]{3})?").unwrap();
    let fr_with_xxx_with_space = Regex::new(r"[A-Z]{4}\s?FR\s?[A-Z0-9]{2}\s?XXX?").unwrap();

    // Helper to get unique matches
    fn get_unique_matches(regex: &Regex, text: &str) -> Vec<String> {
        let matches: Vec<String> = regex
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .unique()
            .collect();
        matches.into_iter().collect()
    }

    let mut fr_without_space_matches = get_unique_matches(&fr_without_space, content);
    log::info!("fr_without_space_matches: {:?}", fr_without_space_matches);
    if fr_without_space_matches.len() == 1 {
        return Some(fr_without_space_matches.pop().unwrap());
    }

    let mut fr_with_xxx_with_space_matches = get_unique_matches(&fr_with_xxx_with_space, content);
    log::info!(
        "fr_with_xxx_with_space_matches: {:?}",
        fr_with_xxx_with_space_matches
    );
    if fr_with_xxx_with_space_matches.len() == 1 {
        return Some(fr_with_xxx_with_space_matches.pop().unwrap());
    }

    // remove all spaces and try again
    let whitespace_regex = Regex::new(r"\s+").unwrap();
    let content_without_spaces = whitespace_regex.replace_all(content, "");
    let mut joined_fr_without_space_matches =
        get_unique_matches(&fr_without_space, &content_without_spaces);
    log::info!(
        "joined_fr_without_space_matches: {:?}",
        joined_fr_without_space_matches
    );
    if joined_fr_without_space_matches.len() == 1 {
        return Some(joined_fr_without_space_matches.pop().unwrap());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_iban() {
        let iban = "FR76 3000 1000 6449 1900 9562 088".to_string();
        assert_eq!(extract_iban(iban.clone()).unwrap(), iban);
    }

    #[test]
    fn test_one_rib() {
        let iban = "FR76 3000 1000 6449 1900 9562 088".to_string();

        fn vec_to_string(v: Vec<&str>) -> Vec<String> {
            v.iter().map(|x| x.to_string()).collect()
        }

        fn to_rib(path: &str) -> Rib {
            let layout_text = std::fs::read_to_string(path).unwrap();
            Rib::try_from(layout_text)
                .unwrap_or_else(|_| panic!("Failed to parse RIB from file: {}", path))
        }

        let path = "tests/fixtures/rib/banque_populaire.txt";
        let titulaire = Some(vec_to_string(vec![
            "M OU MME MATISSE HENRI",
            "51 RUE BERNARD ROY",
            "44100 NANTES",
        ]));
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.clone(),
                bic: Some("BDFEFRPPCCT".to_string())
            }
        );

        let path = "tests/fixtures/rib/banque_populaire_2.txt";
        let titulaire = Some(vec_to_string(vec![
            "M HENRI MATISSE OU MLLE",
            "FRIDA KAHLO",
            "31 AVENUE JULES RENARD",
            "44800 ST HERBLAIN",
        ]));
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.clone(),
                bic: Some("BDFEFRPPCCT".to_string())
            }
        );

        let path = "tests/fixtures/rib/banque_postale.txt";
        let titulaire = Some(vec_to_string(vec![
            "MR MATISSE HENRI",
            "243 RUE DES GRIVES",
            "44240 LA CHAPELLE SUR ERDRE",
        ]));
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.clone(),
                bic: Some("PSSTFRPPNTE".to_string())
            }
        );

        let path = "tests/fixtures/rib/bourso.txt";
        let titulaire = Some(vec_to_string(vec![
            "Mlle Kahlo Frida",
            "55 CHEMIN DU PETIT BOIS",
            "44400 REZE",
        ]));
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.clone(),
                bic: Some("BOUS FRPP XXX".to_string())
            }
        );

        let path = "tests/fixtures/rib/caisse_epargne.txt";
        let titulaire = Some(vec_to_string(vec![
            "MME KAHLO FRIDA OU M MATISSE",
            "143 ALLEE DES SALICAIRES",
            "44240 LA CHAPELLE SUR ERDRE",
        ]));
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.clone(),
                bic: Some("CEPAFRPP444".to_string())
            }
        );

        let path = "tests/fixtures/rib/caisse_epargne_2.txt";
        let titulaire = Some(vec_to_string(vec![
            "M MATISSE HENRI",
            "12 RUE VICTOR FORTUN",
            "44400 REZE",
        ]));
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.clone(),
                bic: Some("CEPAFRPP444".to_string())
            }
        );

        let path = "tests/fixtures/rib/credit_agricole.txt";
        let titulaire = Some(vec_to_string(vec![
            "MR OU MME MATISSE",
            "HENRI",
            "32 RUE EDOUARD TRAVIES",
            "44240 LA CHAPELLE SUR ERDRE",
        ]));
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.clone(),
                bic: Some("AGRIFRPP847".to_string())
            }
        );

        let path = "tests/fixtures/rib/credit_agricole_2.txt";
        let titulaire = Some(vec_to_string(vec!["MME KAHLO FRIDA"]));
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.clone(),
                bic: Some("AGRIFRPP847".to_string())
            }
        );

        let path = "tests/fixtures/rib/credit_mutuel.txt";
        let titulaire = Some(vec_to_string(vec![
            "M HENRI MATISSE",
            "123 ALLEE DES ROSES",
            "44640 LE PELLERIN",
        ]));
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.clone(),
                bic: Some("CMCIFR2A".to_string())
            }
        );

        let path = "tests/fixtures/rib/credit_mutuel_2.txt";
        let titulaire = Some(vec_to_string(vec![
            "M OU MME MATISSE HENRI",
            "54 RUE DE L HERONNIERE",
            "44000 NANTES",
        ]));
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.clone(),
                bic: Some("CMBRFR2BXXX".to_string())
            }
        );

        let path = "tests/fixtures/rib/fortuneo.txt";
        let titulaire = None;
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.clone(),
                bic: Some("FTNOFRP1XXX".to_string())
            }
        );

        let path = "tests/fixtures/rib/sg.txt";
        let titulaire = Some(vec_to_string(vec![
            "Mlle Frida Khalo",
            "117 rue des bourdonnieres 204 batiment c",
            "44200 Nantes",
        ]));
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.clone(),
                bic: Some("SOGEFRPP".to_string())
            }
        );

        let path = "tests/fixtures/rib/sg_2.txt";
        let titulaire = Some(vec_to_string(vec![
            "SAS HENRI MATISSE",
            "18 RUE SADI CARNOT",
            "92120 MONTROUGE",
        ]));
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.clone(),
                bic: Some("SOGEFRPP".to_string())
            }
        );

        let path = "tests/fixtures/rib/orange.txt";
        let titulaire = Some(vec_to_string(vec!["M Matisse Henri"]));
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.clone(),
                bic: Some("GPBAFRPPXXX".to_string())
            }
        );
    }
}
