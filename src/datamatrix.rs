use rxing::{BarcodeFormat, DecodeHintType, DecodeHintValue, DecodingHintDictionary};

pub fn fetch_datamatrix(file_name: &str) -> Option<String> {
    let mut hints = DecodingHintDictionary::from([(
        DecodeHintType::TRY_HARDER,
        DecodeHintValue::TryHarder(true),
    )]);

    rxing::helpers::detect_in_file_with_hints(
        file_name,
        Some(BarcodeFormat::DATA_MATRIX),
        &mut hints,
    )
    .ok()
    .map(|res| res.getText().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let result = "DC02FR000001125E125C0026FR245700010MLLE/SAMPLE/ANGELA\u{1d}20\u{1d}21BAT 2 ETG 3\u{1d}23\u{1d}25METZ\u{1d}227 PLACE DES SPECIMENS\u{1d}\u{1f}Z2HSK7UZM6KPL7UL6OK7NR77GSPGPNNUYYEE4ZV75L5OCIWKVOXTV3I5AJLRSUDOIR76F75QY5Z7KLH3FACKHVF7JH3DYMRI5EIAZMI";

        assert_eq!(
            fetch_datamatrix("tests/fixtures/2ddoc/justificatif_de_domicile.png"),
            Some(result.to_string())
        );
    }
}
