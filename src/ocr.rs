use std::io::Cursor;

use image::{DynamicImage, ImageDecoder, ImageReader};
use log::trace;
use ocrs::TextLine;
use regex::Regex;

use crate::{
    image_utils::{clean_image, only_rotate, resize, rotate, save_image_in_debug},
    ocrs::{extract_anchors, image_to_string_using_ocrs, ocrs_anchors},
    rib::{extract_fr_bic, extract_iban, Rib},
    tesseract::{img_to_string_using_tesseract, tess_analyze},
    text::simple_titulaire::find_simple_titulaire,
};

const OPTIMAL_TESSERACT_HEIGHT: u32 = 30;

pub fn image_bytes_to_rib(content: Vec<u8>, name: &str) -> Option<Rib> {
    let img = bytes_to_img(content)?;
    save_image_in_debug(&img, name, "");

    zoom_and_extract(&img, name).or_else(|| {
        let cleaned_img = clean_image(&img, name);
        zoom_and_extract(&cleaned_img, name)
    })
}

pub fn zoom_and_extract(img: &DynamicImage, name: &str) -> Option<Rib> {
    let mut rib = None;
    let iban_regex = Regex::new(r"(?:^|\s)FR[\dO]").unwrap();

    let (ocrs_text, text_lines, maybe_anchors) = ocrs_anchors(img, &iban_regex, None);
    let maybe_anchor = maybe_anchors.first();

    if let Some(iban) = extract_iban(&ocrs_text) {
        trace!("early returns from ocrs for: {}", name);
        let bic = extract_fr_bic(&ocrs_text);
        let titulaire = zoom_and_extract_titulaire(img, text_lines, name);
        return Some(Rib::from_iban(iban, titulaire, bic));
    };

    if let Some(anchor) = maybe_anchor {
        trace!("ocrs anchor found");

        let iban_image = crop(img, anchor.iban_mask(), name, "mask");
        let iban = extract_iban_in_image(&iban_image, name);

        if let Some(iban) = iban {
            let titulaire = zoom_and_extract_titulaire(img, text_lines.clone(), name);
            let bic = extract_fr_bic(&ocrs_text);
            rib = Some(Rib::from_iban(iban, titulaire, bic));
        } else {
            // maybe this is a long iban with some | between words
            let iban_image = crop(img, anchor.narrow_iban_mask(), name, "narrow_mask");
            let iban = extract_iban_in_image(&iban_image, name);

            if let Some(iban) = iban {
                let titulaire = zoom_and_extract_titulaire(img, text_lines, name);
                let bic = extract_fr_bic(&ocrs_text);
                rib = Some(Rib::from_iban(iban, titulaire, bic));
            }
        }
    }

    if rib.is_some() {
        return rib;
    }

    let (_hocr_string, maybe_angle, maybe_anchor) = tess_analyze(img);

    let (img, maybe_anchor) = maybe_angle
        .map(|angle| {
            let rotated_img = rotate(img, angle);
            let (_, _, new_anchor) = tess_analyze(&rotated_img);
            (rotated_img, new_anchor)
        })
        .unwrap_or((img.clone(), maybe_anchor));

    if let Some(anchor) = maybe_anchor {
        trace!("tess anchor found");

        let iban_image = crop(&img, anchor.iban_mask(), name, "mask");

        let iban_image = only_rotate(&iban_image, name);
        let iban_image = resize(&iban_image, anchor.height, OPTIMAL_TESSERACT_HEIGHT);
        save_image_in_debug(&iban_image, name, "rotated_resized_mask");

        let iban = extract_iban_in_image(&iban_image, name);
        if let Some(iban) = iban {
            let (ocrs_text, text_lines, _) = ocrs_anchors(&img, &iban_regex, None);
            let titulaire = zoom_and_extract_titulaire(&img, text_lines, name);
            let bic = extract_fr_bic(&ocrs_text);
            rib = Some(Rib::from_iban(iban, titulaire, bic));
        }
    }

    rib
}

fn zoom_and_extract_titulaire(
    img: &DynamicImage,
    text_lines: Vec<TextLine>,
    name: &str,
) -> Option<Vec<String>> {
    let code_postal_line_regex = Regex::new(r"[[:space:]]*\d{5}\s+[[:alpha:]]").unwrap();
    let code_postal_word_regex = Regex::new(r"^\d{5}").unwrap();

    fn match_civilite(s: &str) -> bool {
        let civilite =
            Regex::new(r"(?i)(^|\s)(m|monsieur|mr|mademoiselle|ml|mle|mlle|melle|madame|mme)\.?\s")
                .unwrap();
        let prenom_nom_ou =
            Regex::new(r"[[:upper:]]+ +[[:upper:]]+ +OU +[[:upper:]]+ +[[:upper:]]+").unwrap();

        civilite.is_match(s) || prenom_nom_ou.is_match(s)
    }

    fn find_civilite(s: &str) -> Option<usize> {
        let civilite =
            Regex::new(r"(?i)(^|\s)(m|monsieur|mr|mademoiselle|ml|mle|mlle|melle|madame|mme)\.?\s")
                .unwrap();
        let prenom_nom_ou =
            Regex::new(r"[[:upper:]]+ +[[:upper:]]+ +OU +[[:upper:]]+ +[[:upper:]]+").unwrap();
        if civilite.is_match(s) {
            civilite.find(s).map(|m| m.start())
        } else if prenom_nom_ou.is_match(s) {
            prenom_nom_ou.find(s).map(|m| m.start())
        } else {
            None
        }
    }

    let postal_anchors = extract_anchors(
        text_lines,
        &code_postal_word_regex,
        Some(&code_postal_line_regex),
    );

    let titulaires = postal_anchors
        .iter()
        .enumerate()
        .map(|(index, anchor)| {
            let cropped_img = crop(
                img,
                anchor.addr_mask(),
                name,
                &format!(r#"{}_addr_mask"#, index),
            );
            (index, image_to_string_using_ocrs(cropped_img), anchor)
        })
        .filter_map(|(index, text, anchor)| {
            if match_civilite(&text) {
                Some(text)
            } else {
                let cropped_img = crop(
                    img,
                    anchor.right_align_addr_mask(),
                    name,
                    &format!(r#"{}_right_align_addr_mask"#, index),
                );
                let new_text = image_to_string_using_ocrs(cropped_img);

                if match_civilite(&new_text) {
                    Some(new_text)
                } else {
                    None
                }
            }
        })
        .map(|text| {
            // on supprime tout ce qui se situe avant civilite
            let start = find_civilite(&text).unwrap();
            let text = text[start..].trim().to_string();

            // on supprime toutes les lignes situées après le code postal
            let lines: Vec<&str> = text.lines().collect();
            let code_postal_index = lines
                .iter()
                .position(|line| code_postal_line_regex.is_match(line))
                .unwrap();
            lines[..code_postal_index + 1].join("\n")
        })
        .collect::<Vec<String>>();

    let titulaire = titulaires
        .first()
        .map(|s| s.lines().map(|l| l.to_string()).collect());

    if titulaire.is_some() {
        return titulaire;
    }

    let titulaire_word_regex = Regex::new(r"(?i)titulaire").unwrap();
    let (_, _text_lines, titulaire_anchors) = ocrs_anchors(img, &titulaire_word_regex, None);

    titulaire_anchors
        .iter()
        .enumerate()
        .map(|(index, anchor)| {
            let cropped_img = crop(
                img,
                anchor.titulaire_mask(),
                name,
                &format!(r#"{}_titulaire_mask"#, index),
            );
            image_to_string_using_ocrs(cropped_img)
        })
        .filter(|text| titulaire_word_regex.is_match(text))
        .find_map(|text| find_simple_titulaire(&text, 1))
}

fn crop(
    img: &DynamicImage,
    (x, y, width, height): (u32, u32, u32, u32),
    name: &str,
    suffix: &str,
) -> DynamicImage {
    let result = img.crop_imm(x, y, width, height);
    save_image_in_debug(&result, name, suffix);
    result
}

fn bytes_to_img(content: Vec<u8>) -> Option<DynamicImage> {
    let mut decoder = ImageReader::new(Cursor::new(content))
        .with_guessed_format()
        .ok()?
        .into_decoder()
        .ok()?;

    let orientation = decoder.orientation().ok()?;
    let mut img = DynamicImage::from_decoder(decoder).ok()?;
    img.apply_orientation(orientation);
    Some(img.into_luma8().into())
}

fn extract_iban_in_image(cropped_img: &DynamicImage, name: &str) -> Option<String> {
    let tess_iban = img_to_string_using_tesseract(cropped_img.clone());
    if let Some(iban) = extract_iban(&tess_iban) {
        return Some(iban);
    };

    let ocrs_iban = image_to_string_using_ocrs(cropped_img.clone());
    if let Some(iban) = extract_iban(&ocrs_iban) {
        return Some(iban);
    };

    log::trace!(
        "not found for {}: tess_iban: {}, ocrs_iban: {}",
        name,
        tess_iban,
        ocrs_iban
    );

    None
}
