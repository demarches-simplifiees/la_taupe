use nom::{bytes::complete::tag, error::Error, sequence::Tuple};

use self::{
    entete::Entete,
    utils::{date, four_alphanum, two_alphanum},
};

pub mod entete;
pub mod utils;

pub fn parse(i: &str) -> Option<Entete> {
    // DC02 : we only support this version
    let (i, _) = (tag::<&str, &str, Error<&str>>("DC"), tag("02"))
        .parse(i)
        .ok()?;

    let mut v2_2ddoc = (four_alphanum, four_alphanum, date, date, two_alphanum);
    v2_2ddoc
        .parse(i)
        .ok()
        .map(|(_, entete)| Entete::from(entete))
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn test_parse() {
        let i = "DC02FR000001125E125C0026FR245700010MLLE/SAMPLE/ANGELA\u{1d}20\u{1d}21BAT 2 ETG 3\u{1d}23\u{1d}25METZ\u{1d}227 PLACE DES SPECIMENS\u{1d}\u{1f}Z2HSK7UZM6KPL7UL6OK7NR77GSPGPNNUYYEE4ZV75L5OCIWKVOXTV3I5AJLRSUDOIR76F75QY5Z7KLH3FACKHVF7JH3DYMRI5EIAZMI";

        assert_eq!(
            parse(i).unwrap(),
            Entete {
                autorite_certification: "FR00".to_string(),
                identifiant_du_certificat: "0001".to_string(),
                date_emission: NaiveDate::from_ymd_opt(2012, 11, 15).unwrap(),
                date_creation_signature: NaiveDate::from_ymd_opt(2012, 11, 13).unwrap(),
                type_document_id: "00".to_string()
            }
        );
    }
}
