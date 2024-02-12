use std::collections::HashMap;

use self::{
    entete::Entete,
    utils::{date, four_alphanum, two_alphanum, two_digit},
};

use crate::twoddoc::data_structure::data_structure;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::alphanumeric1,
    error::Error,
    multi::many1,
    sequence::{preceded, separated_pair, terminated, Tuple},
    IResult,
};

pub mod data_structure;
pub mod entete;
mod signature;
pub mod utils;

pub fn parse(doc: &str) -> Option<(Entete, HashMap<&str, &str>)> {
    let (i, version) = version(doc)?;

    let (message, entete) = match version {
        2 => { 
            let (m, entete) = (four_alphanum, four_alphanum, date, date, two_alphanum)
                .parse(i).unwrap();

            (m, Entete::from(entete))
        },
        3 => { 
            let (m, entete) = (four_alphanum, four_alphanum, date, date, two_alphanum, two_alphanum)
                .parse(i).unwrap();

            (m, Entete::from(entete))
        },
        _ => {
            log::warn!("Unsupported version: {}", version);
            return None
        }
    };

    let (_, data) = many1(datum)(message).ok()?;
    let bag: HashMap<&str, &str> = data.iter().cloned().collect();

    check_signature(
        doc,
        &entete.autorite_certification,
        &entete.identifiant_du_certificat,
    );

    Some((entete, bag))
}

pub fn version(i: &str) -> Option<(&str, u32)> {
    preceded(tag("DC"), two_digit)(i).ok()
}

fn datum(i: &str) -> IResult<&str, (&str, &str)> {
    let (i, data_id) = two_alphanum(i)?;
    alt((
        terminated(data_structure(data_id), tag("")),
        data_structure(data_id),
        terminated(data_structure(data_id), tag("")), // tronqué
    ))(i)
    .map(|(i, data)| (i, (data_id, data)))
}

fn check_signature(i: &str, autorite_certification: &str, identifiant_du_certificat: &str) {
    let (_, (payload, signature)) = separated_pair(
        is_not::<&str, &str, Error<&str>>(""),
        tag(""),
        alphanumeric1,
    )(i)
    .unwrap();

    signature::check(
        payload,
        signature,
        autorite_certification,
        identifiant_du_certificat,
    );
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    // "<US>"
    // "<GS>"
    // "<RS>"
    #[test]
    fn test_parse_v2_doc_00() {
        let i = "DC02FR000001125E125C0026FR245700010MLLE/SAMPLE/ANGELA2021BAT 2 ETG 32325METZ227 PLACE DES SPECIMENSZ2HSK7UZM6KPL7UL6OK7NR77GSPGPNNUYYEE4ZV75L5OCIWKVOXTV3I5AJLRSUDOIR76F75QY5Z7KLH3FACKHVF7JH3DYMRI5EIAZMI";

        let (entete, bag) = parse(i).unwrap();

        assert_eq!(
            entete,
            Entete {
                autorite_certification: "FR00".to_string(),
                identifiant_du_certificat: "0001".to_string(),
                date_emission: NaiveDate::from_ymd_opt(2012, 11, 15).unwrap(),
                date_creation_signature: NaiveDate::from_ymd_opt(2012, 11, 13).unwrap(),
                type_document_id: "00".to_string(),
                perimetre: None,
            }
        );

        assert_eq!(
            bag,
            HashMap::from([
                ("10", "MLLE/SAMPLE/ANGELA"),
                ("20", ""),
                ("21", "BAT 2 ETG 3"),
                ("22", "7 PLACE DES SPECIMENS"),
                ("23", ""),
                ("24", "57000"),
                ("25", "METZ"),
                ("26", "FR"),
            ])
        );
    }

    #[test]
    fn test_parse_v3_doc_01() {
        env_logger::init();

        let i = "DC03FR000001123F1636010126FR247500010MME/SPECIMEN/NATACHA22145 AVENUE DES SPECIMENSFEDMPW5SO5BNZFYP7FIQUYZFV5H3OF6QERDMOBN7BZ4CC4KVJ4XWUH6EW3CSWILAGLN4XQE6AKHX6RNOI3OXVW6X3IKJASZGL62FBUQ";

        let (entete, bag) = parse(i).unwrap();

        assert_eq!(
            entete,
            Entete {
                autorite_certification: "FR00".to_string(),
                identifiant_du_certificat: "0001".to_string(),
                date_emission: NaiveDate::from_ymd_opt(2012, 10, 15).unwrap(),
                date_creation_signature: NaiveDate::from_ymd_opt(2015, 7, 27).unwrap(),
                type_document_id: "01".to_string(),
                perimetre: Some("01".to_string()),
            }
        );

        assert_eq!(
            bag,
            HashMap::from([
                ("10", "MME/SPECIMEN/NATACHA"),
                ("22", "145 AVENUE DES SPECIMENS"),
                ("24", "75000"),
                ("26", "FR"),
            ])
        );
    }
}
