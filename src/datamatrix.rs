use std::collections::HashSet;

use image::DynamicImage;
use rxing::{
    common::HybridBinarizer, BarcodeFormat, BinaryBitmap, BufferedImageLuminanceSource,
    DecodeHintType, DecodeHintValue, DecodingHintDictionary, MultiFormatReader, Reader,
};

pub fn fetch_datamatrix(img: DynamicImage) -> Option<String> {
    let mut multi_format_reader = MultiFormatReader::default();

    let hints = DecodingHintDictionary::from([
        (DecodeHintType::TRY_HARDER, DecodeHintValue::TryHarder(true)),
        (
            DecodeHintType::POSSIBLE_FORMATS,
            DecodeHintValue::PossibleFormats(HashSet::from([BarcodeFormat::DATA_MATRIX])),
        ),
    ]);

    let result = multi_format_reader
        .decode_with_hints(
            &mut BinaryBitmap::new(HybridBinarizer::new(BufferedImageLuminanceSource::new(img))),
            &hints,
        )
        .ok()?;

    Some(result.getText().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_datamatrix() {
        let result = "DC02FR000001125E125C0026FR245700010MLLE/SAMPLE/ANGELA\u{1d}20\u{1d}21BAT 2 ETG 3\u{1d}23\u{1d}25METZ\u{1d}227 PLACE DES SPECIMENS\u{1d}\u{1f}Z2HSK7UZM6KPL7UL6OK7NR77GSPGPNNUYYEE4ZV75L5OCIWKVOXTV3I5AJLRSUDOIR76F75QY5Z7KLH3FACKHVF7JH3DYMRI5EIAZMI";

        let img = image::open("tests/fixtures/2ddoc/justificatif_de_domicile.png").unwrap();

        assert_eq!(fetch_datamatrix(img), Some(result.to_string()));
    }
}
