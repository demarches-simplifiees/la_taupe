use iban::Iban;
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    fi_extract::IbanToBankName,
    text::{address::find_titulaire_addr, simple_titulaire::find_simple_titulaire},
};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Rib {
    titulaire: Option<Vec<String>>,
    iban: String,
    bic: Option<String>,
    bank_name: Option<String>,
}

impl Rib {
    pub fn from_iban(iban: String, titulaire: Option<Vec<String>>, bic: Option<String>) -> Self {
        let bank_name = IbanToBankName::new().bank_name(&iban);

        Rib {
            titulaire,
            iban,
            bic,
            bank_name,
        }
    }
    pub fn parse(text: String) -> Option<Self> {
        let titulaire = find_titulaire_addr(&text)
            .map(|addr| addr.lines())
            .or_else(|| find_simple_titulaire(&text, 3));

        let bic = extract_fr_bic(&text);

        if let Some(iban) = extract_iban(&text) {
            let bank_name = IbanToBankName::new().bank_name(&iban);

            Some(Rib {
                titulaire,
                iban,
                bic,
                bank_name,
            })
        } else {
            None
        }
    }
}

pub fn replace_char_by_digit_in_2_and_3_position(ibans: Vec<String>) -> Vec<String> {
    ibans
        .into_iter()
        .map(|x| {
            let mut chars: Vec<char> = x.chars().collect();
            if chars.len() > 2 && (chars[2] == 'O' || chars[2] == 'o') {
                chars[2] = '0';
            }
            if chars.len() > 3 && (chars[3] == 'O' || chars[3] == 'o') {
                chars[3] = '0';
            }
            chars.into_iter().collect()
        })
        .collect()
}

pub fn extract_iban(text: &str) -> Option<String> {
    let french_iban_re = Regex::new(r"(?<iban>FR[[[:digit:]]O]{2}([[[:space:]]\|,]*[[:alnum:]]{4}){5})([[[:space:]]|,]*[[:alnum:]][[:digit:]]{2})").unwrap();

    let to_remove = Regex::new(r"[[[:space:]]|,]*").unwrap();

    let mut ibans = french_iban_re
        .find_iter(text)
        .map(|x| x.as_str().to_string())
        // change multiple spaces by one space
        .map(|x| to_remove.replace_all(&x, "").to_string())
        .collect::<Vec<String>>();

    ibans = replace_char_by_digit_in_2_and_3_position(ibans);

    let found_ibans = ibans
        .clone()
        .into_iter()
        .filter_map(|x| x.parse::<Iban>().ok())
        .collect::<Vec<Iban>>();

    if !found_ibans.is_empty() {
        return Some(found_ibans[0].to_string());
    }

    // sometimes the iban is written with weird space (credit_agricole_2.txt)
    // so we try to match by removing all spaces
    let text_without_spaces = text.replace(" ", "");
    ibans = french_iban_re
        .find_iter(&text_without_spaces)
        .map(|x| x.as_str().to_string())
        .map(|x| to_remove.replace_all(&x, "").to_string())
        .collect::<Vec<String>>();

    ibans = replace_char_by_digit_in_2_and_3_position(ibans);

    let found_ibans = ibans
        .clone()
        .into_iter()
        .filter_map(|x| x.parse::<Iban>().ok())
        .collect::<Vec<Iban>>();

    if !found_ibans.is_empty() {
        return Some(found_ibans[0].to_string());
    }

    let lax_frenc_iban_re = Regex::new(r"(?<iban>FR[[:alnum:]]{2}([[[:space:]]\|]*[[:alnum:]]{4}){5})([[[:space:]]|]*[[:alnum:]][[:digit:]]{2})").unwrap();

    let lax_ibans = lax_frenc_iban_re
        .find_iter(text)
        .map(|x| x.as_str().to_string())
        // change multiple spaces by one space
        .map(|x| to_remove.replace_all(&x, "").to_string())
        .collect::<Vec<String>>();

    if lax_ibans.len() < 2 {
        return None;
    }

    // we take the 2 first iban and count the number of different characters
    let iban1 = lax_ibans[0].clone();
    let iban2 = lax_ibans[1].clone();

    let mut differences = Vec::new();

    // Itérer sur les caractères des chaînes
    for (index, (c1, c2)) in iban1.chars().zip(iban2.chars()).enumerate() {
        if c1 != c2 {
            differences.push(index);
        }
    }

    // to many combinations
    if differences.len() > 10 {
        return None;
    }

    let mut combinations = Vec::new();
    let num_differences = differences.len();
    let num_combinations = 1 << num_differences; // 2^n

    for i in 0..num_combinations {
        let mut combo = iban1.to_string();
        for (j, &diff_pos) in differences.iter().enumerate() {
            if (i >> j) & 1 == 1 {
                let c = iban2.chars().nth(diff_pos).unwrap();
                combo.replace_range(diff_pos..diff_pos + 1, &c.to_string());
            }
        }
        combinations.push(combo);
    }

    let found_ibans = combinations
        .into_iter()
        .filter_map(|x| x.parse::<Iban>().ok())
        .collect::<Vec<Iban>>();

    if found_ibans.len() == 1 {
        Some(found_ibans[0].to_string())
    } else {
        None
    }
}

pub fn extract_fr_bic(content: &str) -> Option<String> {
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
    log::trace!("fr_without_space_matches: {:?}", fr_without_space_matches);
    if fr_without_space_matches.len() == 1 {
        return Some(fr_without_space_matches.pop().unwrap());
    }

    let mut fr_with_xxx_with_space_matches = get_unique_matches(&fr_with_xxx_with_space, content);
    log::trace!(
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
    log::trace!(
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
        let iban = "FR76 3000 1000 6449 1900 9562 088";
        assert_eq!(extract_iban(iban).unwrap(), iban);

        let other_iban = "FR76 | 3000

          1000 | 6449

          1900 | 9562 | 088";
        assert_eq!(extract_iban(other_iban).unwrap(), iban);

        let iban_with_faults = "
          FRTS 3000 1000 6449 1900 9562 088
          FR76 3000 BOO0 6666 1900 9562 088
        ";

        assert_eq!(extract_iban(iban_with_faults).unwrap(), iban);
    }

    fn vec_to_string(v: Vec<&str>) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    fn to_rib(path: &str) -> Rib {
        let layout_text = std::fs::read_to_string(path).unwrap();
        Rib::parse(layout_text).unwrap_or_else(|| {
            panic!("Failed to parse RIB from file: {}", path);
        })
    }

    fn test_file(path: &str, titulaire: Option<Vec<&str>>, iban: &str, bic: &str) {
        let titulaire = titulaire.map(vec_to_string);
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.to_string(),
                bic: Some(bic.to_string()),
                bank_name: None
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
        let titulaire = Some(vec!["MLE FRIDA KHALO", "OU MR MATISSE HENRI"]);
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
