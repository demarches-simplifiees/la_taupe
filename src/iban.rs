use iban::Iban;
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Rib {
    titulaire: Option<Vec<String>>,
    iban: String,
}

impl TryFrom<String> for Rib {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        let lines = clean(text.clone());

        let titulaire = extract_titulaire(lines.clone());

        if let Some(iban) = extract_iban(text) {
            Ok(Rib { titulaire, iban })
        } else {
            Err(format!("No IBAN found in the text. Lines: {:?}", lines))
        }
    }
}

type Block = Vec<String>;

fn clean(text: String) -> Vec<String> {
    let lines: Vec<String> = text
        .split('\n')
        .map(|line| line.trim_end().to_string()) // Enlève les espaces bizzare en fin de ligne
        .collect();

    let dedup_blocks: Vec<Block> = lines
        // on regroupe les lignes séparées par des lignes vides en bloc
        .split(|line| line.is_empty())
        // on enlève les blocs vides
        .filter(|block| !block.is_empty())
        // on enlève les doublons
        .unique()
        .map(|block| block.to_vec())
        .collect();

    // on merge les blocs consecutifs qui ont la meme position de 2eme colonne
    let merged_blocks: Vec<Block> = dedup_blocks.iter().fold(vec![], |mut acc, block| {
        match (
            acc.last().and_then(second_column_position),
            second_column_position(block),
        ) {
            (Some(last_position), Some(position)) if last_position == position => {
                let mut last_block = acc.pop().unwrap();
                last_block.extend(block.clone());
                acc.push(last_block);
                acc
            }
            _ => {
                acc.push(block.clone());
                acc
            }
        }
    });

    // on resplite les blocs separes par des :
    let splitted_blocks: Vec<Block> = merged_blocks.iter().flat_map(split_left_right).collect();

    // on essaye de detecter les blocs qui contiennent plusieurs colonnes
    let splitted_columns: Vec<Block> = splitted_blocks
        .iter()
        .flat_map(|block: &Block| split_2_columns(block))
        // on enlève les doublons
        .unique()
        .collect();

    // on remet a plat les lignes
    splitted_columns
        .iter()
        .flat_map(|block| block.to_vec())
        .collect()
}

fn second_column_start_position(line: &str, middle: usize) -> Option<usize> {
    let three_spaces = Regex::new(r"\s{3,}").unwrap();

    // si il y a 2 fois ou plus 3 espace dans la ligne, alors ce n est pas en 2 colonnes
    if three_spaces.find_iter(line.trim()).count() > 1 {
        return None;
    }

    line.rfind("   ").map(|pos| pos + 3).map(|pos| {
        // on regarde le barycentere de la ligne et on regarde ou il est mis
        if middle <= pos + (line.trim().len() / 2) {
            Some(pos)
        } else {
            None
        }
    })?
}

fn second_column_position(block: &Block) -> Option<usize> {
    let max_length = block.iter().map(|line| line.len()).max().unwrap_or(0);
    let position: Option<usize> = block
        .iter()
        .filter_map(|line| second_column_start_position(line, max_length / 2))
        .min();
    position
}

fn split_2_columns(block: &Block) -> Vec<Block> {
    let position = second_column_position(block);

    match position {
        Some(pos) => {
            let (first_column, second_column) = block
                .iter()
                .map(|line| {
                    if line.len() < pos {
                        (line.as_str(), "")
                    } else {
                        line.split_at(pos)
                    }
                })
                .fold((vec![], vec![]), |mut acc, (first, second)| {
                    if !first.trim().is_empty() {
                        acc.0.push(first.trim().to_string());
                    }
                    if !second.trim().is_empty() {
                        acc.1.push(second.trim().to_string());
                    }
                    acc
                });

            vec![first_column, second_column]
        }
        None => vec![block.to_vec()],
    }
}

fn split_left_right(block: &Block) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    let mut current_block: Block = Vec::new();

    for line in block {
        if line.contains(':') {
            blocks.push(current_block);
            current_block = Vec::new();
        }
        current_block.push(line.to_string());
    }
    blocks.push(current_block);

    // on enleve les blocs vides
    blocks
        .into_iter()
        .filter(|block| !block.is_empty())
        .collect()
}

fn extract_titulaire(lines: Vec<String>) -> Option<Vec<String>> {
    let titulaire = Regex::new(r"(?i)titulaire|intitulé du compte").unwrap();
    let code_postal = Regex::new(r"^\d{5}").unwrap();

    let titulaire_index = lines.iter().position(|x| titulaire.is_match(x));

    let code_postal_index = lines[titulaire_index?..]
        .iter()
        .position(|x| code_postal.is_match(x));

    match (titulaire_index, code_postal_index) {
        (Some(titulaire_index), Some(code_postal_index)) => {
            let titulaire = lines[(titulaire_index + 1)..=(code_postal_index + titulaire_index)]
                .iter()
                // on enleve ": " en debut de ligne
                .map(|x| x.trim_start_matches(": ").to_string())
                .collect::<Vec<String>>();
            Some(titulaire)
        }
        _ => None,
    }
}

fn extract_iban(text: String) -> Option<String> {
    // FR56 2004 1010 1114 6801 6D03 279
    // max 34 char = 8*4 +2 = entete + 7*4 + 2 = entete + 3*4 (<- min belge) + 4 * 4 + 2
    // [[:digit:]O] because of OCR error
    let iban_re = Regex::new(r"(?<iban>[[:upper:]]{2}[[:digit:]O]{2}([[:space:]]+[[:alnum:]]{4}){3,7}([[:space:]]+[[:alnum:]]{0,4}){0,1})").unwrap();

    let iban = iban_re
        .find(&text)
        .map(|x| x.as_str().to_string())
        // change multiple spaces by one space
        .map(|x| x.split_whitespace().collect::<Vec<&str>>().join(" "));

    // replace O by 0 in the 3rd and 4th position
    // to match the iban format
    // TODO: should be able to add more rules to clean the iban
    // depending on the country code
    let cleaned_iban = iban.map(|x| {
        x.chars()
            .enumerate()
            .map(|(i, c)| {
                if (i == 2 || i == 3) && c == 'O' {
                    '0'
                } else {
                    c
                }
            })
            .collect::<String>()
    });

    cleaned_iban?.parse::<Iban>().ok().map(|x| x.to_string())
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    #[test]
    fn test_extract_iban() {
        let iban = "FR76 3000 1000 6449 1900 9562 088".to_string();
        assert_eq!(extract_iban(iban.clone()).unwrap(), iban);

        // OCR error, O instead of 0 in the 3rd and 4th position
        let weird_iban = "FRO5 2004 1000 0121 0436 2F02 027".to_string();
        assert_eq!(
            extract_iban(weird_iban),
            Some("FR05 2004 1000 0121 0436 2F02 027".to_string())
        );
    }

    #[test]
    fn test_second_column_start_position() {
        let line = "first column";
        assert_eq!(second_column_start_position(line, line.len() / 2), None);

        let line = "  first column";
        assert_eq!(second_column_start_position(line, line.len() / 2), None);

        let line = "                second column";
        assert_eq!(second_column_start_position(line, line.len() / 2), Some(16));

        let line = "first column   second column";
        assert_eq!(second_column_start_position(line, line.len() / 2), Some(15));

        let line = "first column        second column";
        assert_eq!(second_column_start_position(line, line.len() / 2), Some(20));

        let line = "first column        second column";
        assert_eq!(second_column_start_position(line, line.len() / 2), Some(20));

        let line = "first column        second column      third column";
        assert_eq!(second_column_start_position(line, line.len() / 2), None);

        let line = "   first column        second column";
        assert_eq!(second_column_start_position(line, line.len() / 2), Some(23));

        // TODO
        // let line = "first column        second column   ";
        // assert_eq!(second_column_start_position(line, line.len()/2), Some(20));
    }

    #[test]
    fn test_split_left_right() {
        let lines = vec![
            "titulaire               : M. Rene Coty".to_string(),
            "                          51 rue du patelin".to_string(),
            "                          38600 par la bas".to_string(),
            "tel                      : 06 06 06 06 06".to_string(),
        ];

        println!("{:?}", split_left_right(&lines));
    }

    #[test]
    fn test_split_2_columns() {
        let lines = vec!["first column".to_string()];
        assert_eq!(split_2_columns(&lines), vec![vec!["first column"]]);

        let lines = vec!["first column   second column".to_string()];
        assert_eq!(
            split_2_columns(&lines),
            vec![vec!["first column"], vec!["second column"]]
        );

        // second column left align
        let lines = vec![
            "once upon   in a galaxy".to_string(),
            "a time      far far away".to_string(),
        ];

        let expected = vec![
            vec!["once upon", "a time"],
            vec!["in a galaxy", "far far away"],
        ];

        assert_eq!(split_2_columns(&lines), expected);

        // second column right align
        let lines = vec![
            "once upon    in a galaxy".to_string(),
            "a time      far far away".to_string(),
        ];

        let expected = vec![
            vec!["once upon", "a time"],
            vec!["in a galaxy", "far far away"],
        ];

        assert_eq!(split_2_columns(&lines), expected);

        let lines = vec![
            "once upon".to_string(),
            "a time      in a galaxy".to_string(),
        ];

        let expected = vec![vec!["once upon", "a time"], vec!["in a galaxy"]];

        assert_eq!(split_2_columns(&lines), expected);
    }

    #[allow(dead_code)]
    fn read_file_using_poppler(path: &str) -> String {
        let output = Command::new("pdftotext")
            .arg("-layout")
            .arg(path)
            .arg("-")
            .output()
            .expect("Failed to execute command");

        String::from_utf8_lossy(&output.stdout).to_string()
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
                iban: iban.clone()
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
                iban: iban.clone()
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
                iban: iban.clone()
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
                iban: iban.clone()
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
                iban: iban.clone()
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
                iban: iban.clone()
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
                iban: iban.clone()
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
                iban: iban.clone()
            }
        );

        let path = "tests/fixtures/rib/fortuneo.txt";
        let titulaire = None;
        assert_eq!(
            to_rib(path),
            Rib {
                titulaire,
                iban: iban.clone()
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
                iban: iban.clone()
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
                iban: iban.clone()
            }
        );
    }
}
