use std::collections::HashMap;

use self::{
    ddoc::Ddoc,
    entete::Entete,
    utils::{date, date_option, four_alphanum, two_alphanum, two_digit},
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

mod certificate_store;
pub mod data_structure;
pub mod ddoc;
pub mod entete;
mod signature;
pub mod trust_service;
pub mod utils;

pub fn parse(doc: &str) -> Option<Ddoc> {
    let (i, version) = version(doc)?;

    let (message, entete) = match version {
        2 => {
            let (m, entete) = (
                four_alphanum,
                four_alphanum,
                date_option,
                date,
                two_alphanum,
            )
                .parse(i)
                .unwrap();

            (m, Entete::from(entete))
        }
        3 => {
            let (m, entete) = (
                four_alphanum,
                four_alphanum,
                date_option,
                date,
                two_alphanum,
                two_alphanum,
            )
                .parse(i)
                .unwrap();

            (m, Entete::from(entete))
        }
        4 => {
            let (m, entete) = (
                four_alphanum,
                four_alphanum,
                date_option,
                date,
                two_alphanum,
                two_alphanum,
                two_alphanum,
            )
                .parse(i)
                .unwrap();

            (m, Entete::from(entete))
        }
        _ => {
            log::warn!("Unsupported version: {}", version);
            return None;
        }
    };

    let (_, data) = many1(datum)(message).ok()?;
    let bag: HashMap<String, String> = data
        .iter()
        .cloned()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    check_signature(
        doc,
        &entete.autorite_certification,
        &entete.identifiant_du_certificat,
    );

    Some(Ddoc::new(entete, bag))
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
    use chrono::{NaiveDate, NaiveDateTime};

    use super::*;

    fn date_time_from(year: i32, month: u32, day: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    }

    fn data_from(kvs: &[(&str, &str)]) -> HashMap<String, String> {
        kvs.iter()
            .cloned()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    // "<US>"
    // "<GS>"
    // "<RS>"
    #[test]
    fn test_parse_v2_doc_00() {
        let i = "DC02FR000001125E125C0026FR245700010MLLE/SAMPLE/ANGELA2021BAT 2 ETG 32325METZ227 PLACE DES SPECIMENSZ2HSK7UZM6KPL7UL6OK7NR77GSPGPNNUYYEE4ZV75L5OCIWKVOXTV3I5AJLRSUDOIR76F75QY5Z7KLH3FACKHVF7JH3DYMRI5EIAZMI";

        let ddoc = parse(i).unwrap();

        assert_eq!(
            ddoc.entete,
            Entete {
                autorite_certification: "FR00".to_string(),
                identifiant_du_certificat: "0001".to_string(),
                date_emission: Some(date_time_from(2012, 11, 15)),
                date_creation_signature: date_time_from(2012, 11, 13),
                type_document_id: "00".to_string(),
                perimetre: None,
                emetteur: None,
            }
        );

        assert_eq!(
            ddoc.data,
            data_from(&[
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
        let i = "DC03FR000001123F1636010126FR247500010MME/SPECIMEN/NATACHA22145 AVENUE DES SPECIMENSFEDMPW5SO5BNZFYP7FIQUYZFV5H3OF6QERDMOBN7BZ4CC4KVJ4XWUH6EW3CSWILAGLN4XQE6AKHX6RNOI3OXVW6X3IKJASZGL62FBUQ";

        let ddoc = parse(i).unwrap();

        assert_eq!(
            ddoc.entete,
            Entete {
                autorite_certification: "FR00".to_string(),
                identifiant_du_certificat: "0001".to_string(),
                date_emission: Some(date_time_from(2012, 10, 15)),
                date_creation_signature: date_time_from(2015, 7, 27),
                type_document_id: "01".to_string(),
                perimetre: Some("01".to_string()),
                emetteur: None,
            }
        );

        assert_eq!(
            ddoc.data,
            data_from(&[
                ("10", "MME/SPECIMEN/NATACHA"),
                ("22", "145 AVENUE DES SPECIMENS"),
                ("24", "75000"),
                ("26", "FR"),
            ])
        );
    }

    #[test]
    fn test_parse_v4_doc_04() {
        let i = "DC04FR000001FFFF1FB60401FR432,7544227801234567845202146RETI PATRICK4A31072022416319847300112345678948RETI SOPHIE490701987765432QHA4A6QOV6AZJEBTIUNR7QOBXINNTMZTD5COQH6VN24NCZTXA7MYXB6SNSNTWAQRYK3ZFP4ZWBGLTJ6SDSPMURF7YFILKQFIAJY7NTI";

        let ddoc = parse(i).unwrap();

        assert_eq!(
            ddoc.entete,
            Entete {
                autorite_certification: "FR00".to_string(),
                identifiant_du_certificat: "0001".to_string(),
                date_emission: None,
                date_creation_signature: date_time_from(2022, 3, 24),
                type_document_id: "04".to_string(),
                perimetre: Some("01".to_string()),
                emetteur: Some("FR".to_string()),
            }
        );

        assert_eq!(
            ddoc.data,
            data_from(&[
                ("43", "2,75"),
                ("44", "2278012345678"),
                ("45", "2021"),
                ("46", "RETI PATRICK"),
                ("4A", "31072022"),
                ("41", "63198"),
                ("47", "3001123456789"),
                ("48", "RETI SOPHIE"),
                ("49", "0701987765432"),
            ])
        );
    }
}
