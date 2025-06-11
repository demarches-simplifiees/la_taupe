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
    let mut maybe_titulaire: Option<Vec<String>> = None;

    let titulaire = Regex::new(r"(?i)titulaire|intitulé du compte").unwrap();
    let code_postal = Regex::new(r"^\d{5}").unwrap();
    let stop_words = Regex::new(r"(?i)domiciliation|identification|iban|cadre réservé|numéro de compte").unwrap();
    let civilite =
        Regex::new(r"(?i)(^|\s)(m|monsieur|mr|mademoiselle|ml|mle|mlle|madame|mme)\.?\s").unwrap();

    let titulaire_index = lines.iter().position(|x| titulaire.is_match(x));

    if let Some(titulaire_index) = titulaire_index {
        let mut titulaire = Vec::<String>::new();

        // if the titulaire line includes a civilite, we extract it
        // ex : "Titulaire : Mlle Frida Kahlo"
        // ex : "Intitulé du compte ML KHALO FRIDA OU M MATISSE H"
        if let Some(found) = civilite.find(&lines[titulaire_index]) {
            let start_pos = found.start();
            let first_line = lines[titulaire_index][start_pos..].trim().to_string();
            titulaire.push(first_line);
        }

        let code_postal_index = lines[titulaire_index..]
            .iter()
            .position(|x| code_postal.is_match(x));

        let stop_words_index = lines[titulaire_index..]
            .iter()
            .position(|x| stop_words.is_match(x))
            .map(|index| index - 1); // -1 because we exclude the domiciliation line itself

        let end_index = [code_postal_index, stop_words_index]
            .iter()
            .filter_map(|&x| x)
            .min();

        if let Some(end_index) = end_index {
            let mut more_titulaire = lines
                [(titulaire_index + 1)..=(end_index + titulaire_index)]
                .iter()
                    // remove lines containing only 'word :' as 'Compte :'
                    .filter(|x| !regex::Regex::new(r"^\s*\w+\s*:\s*$").unwrap().is_match(x))
                    // remove ": " at the beginning of the line
                    .map(|x| x.trim_start_matches(": ").to_string())
                    .collect::<Vec<String>>();

            titulaire.append(&mut more_titulaire);
        }

        if titulaire.is_empty() {
            maybe_titulaire = None;
        } else {
            maybe_titulaire = Some(titulaire);
        };
    } else {
        let civilite_index = lines
            .iter()
            .position(|x| civilite.is_match(x));

        let code_postal_index = lines[civilite_index?..]
            .iter()
            .position(|x| code_postal.is_match(x));

        let stop_word_index = lines[civilite_index?..]
            .iter()
            .position(|x| stop_words.is_match(x))
            .map(|index| index - 1); // -1 because we exclude the domiciliation line itself

        let end_index = [code_postal_index, stop_word_index]
            .iter()
            .filter_map(|&x| x)
            .min();

        if end_index.is_some() {
            let titulaire = lines[civilite_index?..=(end_index.unwrap() + civilite_index?)]
                .iter()
                .map(|x| x.trim_start_matches(": ").to_string())
                .collect::<Vec<String>>();

            maybe_titulaire = Some(titulaire);
        }
    }

    maybe_titulaire
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

    // try known banks BICs
    let caisse_epargne_bic = Regex::new(r"CEPAFRPP[A-Z0-9]{3}").unwrap();
    let mut caisse_epargne_bic_matches =
        get_unique_matches(&caisse_epargne_bic, &content_without_spaces);
    if caisse_epargne_bic_matches.len() == 1 {
        return Some(caisse_epargne_bic_matches.pop().unwrap());
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

    fn vec_to_string(v: Vec<&str>) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    fn to_rib(path: &str) -> Rib {
        let layout_text = std::fs::read_to_string(path).unwrap();
        Rib::try_from(layout_text)
            .unwrap_or_else(|_| panic!("Failed to parse RIB from file: {}", path))
    }

    fn test_file(path: &str, titulaire: Option<Vec<&str>>, iban: &str, bic: &str) {
        let titulaire = titulaire.map(vec_to_string);
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.to_string(),
                bic: Some(bic.to_string())
            }
        );
    }

    static IBAN: &str = "FR76 3000 1000 6449 1900 9562 088";

    #[test]
    fn rib_banque_populaire() {
        let path = "tests/fixtures/rib/banque_populaire.txt";
        let titulaire = Some(vec![
            "M OU MME MATISSE HENRI",
            "51 RUE BERNARD ROY",
            "44100 NANTES",
        ]);
        let bic = "BDFEFRPPCCT";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_banque_populaire_2() {
        let path = "tests/fixtures/rib/banque_populaire_2.txt";
        let titulaire = Some(vec![
            "M HENRI MATISSE OU MLLE",
            "FRIDA KAHLO",
            "31 AVENUE JULES RENARD",
            "44800 ST HERBLAIN",
        ]);
        let bic = "BDFEFRPPCCT";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_banque_postale() {
        let path = "tests/fixtures/rib/banque_postale.txt";
        let titulaire = Some(vec![
            "MR MATISSE HENRI",
            "243 RUE DES GRIVES",
            "44240 LA CHAPELLE SUR ERDRE",
        ]);
        let bic = "PSSTFRPPNTE";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_banque_postale_2() {
        let path = "tests/fixtures/rib/banque_postale_2.txt";
        let titulaire = Some(vec![
            "MLE FRIDA KHALO",
            "OU MR MATISSE HENRI",
        ]);
        let bic = "PSSTFRPPNTE";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_bourso() {
        let path = "tests/fixtures/rib/bourso.txt";
        let titulaire = Some(vec![
            "Mlle Kahlo Frida",
            "55 CHEMIN DU PETIT BOIS",
            "44400 REZE",
        ]);
        let bic = "BOUS FRPP XXX";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_caisse_epargne() {
        let path = "tests/fixtures/rib/caisse_epargne.txt";
        let titulaire = Some(vec![
            "MME KAHLO FRIDA OU M MATISSE",
            "143 ALLEE DES SALICAIRES",
            "44240 LA CHAPELLE SUR ERDRE",
        ]);
        let bic = "CEPAFRPP444";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_caisse_epargne_2() {
        let path = "tests/fixtures/rib/caisse_epargne_2.txt";
        let titulaire = Some(vec![
            "M MATISSE HENRI",
            "12 RUE VICTOR FORTUN",
            "44400 REZE",
        ]);
        let bic = "CEPAFRPP444";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_caisse_epargne_3() {
        let path = "tests/fixtures/rib/caisse_epargne_3.txt";
        let titulaire = Some(vec![
            "ML KHALO FRIDA OU M MATISSE H",
            "35 RUE DU CEDRE",
            "44240 LA CHAPELLE SUR ERDRE",
        ]);
        let bic = "CEPAFRPP444";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_credit_agricole() {
        let path = "tests/fixtures/rib/credit_agricole.txt";
        let titulaire = Some(vec![
            "MR OU MME MATISSE",
            "HENRI",
            "32 RUE EDOUARD TRAVIES",
            "44240 LA CHAPELLE SUR ERDRE",
        ]);
        let bic = "AGRIFRPP847";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_credit_agricole_2() {
        let path = "tests/fixtures/rib/credit_agricole_2.txt";
        let titulaire = Some(vec!["MME KAHLO FRIDA"]);
        let bic = "AGRIFRPP847";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_credit_agricole_3() {
        let path = "tests/fixtures/rib/credit_agricole_3.txt";
        let titulaire = Some(vec![
            "MLLE FRIDA KHALO",
            "15 RUE MARYSE BASTIE",
            "44230 ST SEBASTIEN SUR LOIRE",
        ]);
        let bic = "AGRIFRPP847";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_credit_mutuel() {
        let path = "tests/fixtures/rib/credit_mutuel.txt";
        let titulaire = Some(vec![
            "M HENRI MATISSE",
            "123 ALLEE DES ROSES",
            "44640 LE PELLERIN",
        ]);
        let bic = "CMCIFR2A";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_credit_mutuel_2() {
        let path = "tests/fixtures/rib/credit_mutuel_2.txt";
        let titulaire = Some(vec![
            "M OU MME MATISSE HENRI",
            "54 RUE DE L HERONNIERE",
            "44000 NANTES",
        ]);
        let bic = "CMBRFR2BXXX";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_fortuneo() {
        let path = "tests/fixtures/rib/fortuneo.txt";
        let titulaire = Some(vec!["Madame Khalo Frida ou Monsieur Matisse Henri"]);
        let bic = "FTNOFRP1XXX";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_lcl() {
        let path = "tests/fixtures/rib/lcl.txt";
        let titulaire = Some(vec!["M MATISSE HENRI"]);
        let bic = "CRLYFRPP";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_lcl_2() {
        let path = "tests/fixtures/rib/lcl_2.txt";
        let titulaire = Some(vec!["MLLE FRIDA KHALO"]);
        let bic = "CRLYFRPP";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_sg() {
        let path = "tests/fixtures/rib/sg.txt";
        let titulaire = Some(vec![
            "Mlle Frida Khalo",
            "117 rue des bourdonnieres 204 batiment c",
            "44200 Nantes",
        ]);
        let bic = "SOGEFRPP";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_sg_2() {
        let path = "tests/fixtures/rib/sg_2.txt";
        let titulaire = Some(vec![
            "SAS HENRI MATISSE",
            "18 RUE SADI CARNOT",
            "92120 MONTROUGE",
        ]);
        let bic = "SOGEFRPP";
        test_file(path, titulaire, IBAN, bic);
    }

    #[test]
    fn rib_orange() {
        let path = "tests/fixtures/rib/orange.txt";
        let titulaire = Some(vec!["M Matisse Henri"]);
        let bic = "GPBAFRPPXXX";
        test_file(path, titulaire, IBAN, bic);
    }
}
