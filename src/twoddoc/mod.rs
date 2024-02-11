use std::collections::HashMap;

use self::{
    entete::Entete,
    utils::{date, four_alphanum, two_alphanum, two_digit},
};

use crate::twoddoc::data_structure::data_structure;
use nom::{
    branch::alt,
    bytes::complete::tag,
    multi::many1,
    sequence::{preceded, terminated, Tuple},
    IResult,
};

pub mod data_structure;
pub mod entete;
pub mod utils;

pub fn parse(i: &str) -> Option<(Entete, HashMap<&str, &str>)> {
    let (i, version) = version(i)?;

    if version != 2 {
        return None;
    }

    let mut v2_2ddoc = (four_alphanum, four_alphanum, date, date, two_alphanum);
    let (message, entete) = v2_2ddoc.parse(i).unwrap();

    let (_, data2) = many1(datum)(message).ok()?;
    let bag: HashMap<&str, &str> = data2.iter().cloned().collect();

    Some((Entete::from(entete), bag))
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

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn test_parse() {
        let i = "DC02FR000001125E125C0026FR245700010MLLE/SAMPLE/ANGELA\u{1d}20\u{1d}21BAT 2 ETG 3\u{1d}23\u{1d}25METZ\u{1d}227 PLACE DES SPECIMENS\u{1d}\u{1f}Z2HSK7UZM6KPL7UL6OK7NR77GSPGPNNUYYEE4ZV75L5OCIWKVOXTV3I5AJLRSUDOIR76F75QY4Z7KLH3FACKHVF7JH3DYMRI5EIAZMI";

        let (entete, bag) = parse(i).unwrap();

        assert_eq!(
            entete,
            Entete {
                autorite_certification: "FR00".to_string(),
                identifiant_du_certificat: "0001".to_string(),
                date_emission: NaiveDate::from_ymd_opt(2012, 11, 15).unwrap(),
                date_creation_signature: NaiveDate::from_ymd_opt(2012, 11, 13).unwrap(),
                type_document_id: "00".to_string()
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
}
