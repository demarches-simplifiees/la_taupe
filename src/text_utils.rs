use itertools::Itertools;
use regex::Regex;

type Block = Vec<String>;

pub fn clean(text: String) -> Vec<String> {
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
        if middle <= pos + (line.trim().chars().count() / 2) {
            Some(pos)
        } else {
            None
        }
    })?
}

fn second_column_position(block: &Block) -> Option<usize> {
    let max_length = block
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
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
                    if line.chars().count() < pos {
                        (line.to_string(), "".to_string())
                    } else {
                        let chars: Vec<char> = line.chars().collect();
                        (
                            chars[..pos].iter().collect::<String>(),
                            chars[pos..].iter().collect::<String>(),
                        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_second_column_start_position() {
        let line = "first column";
        assert_eq!(
            second_column_start_position(line, line.chars().count() / 2),
            None
        );

        let line = "  first column";
        assert_eq!(
            second_column_start_position(line, line.chars().count() / 2),
            None
        );

        let line = "                second column";
        assert_eq!(
            second_column_start_position(line, line.chars().count() / 2),
            Some(16)
        );

        let line = "first column   second column";
        assert_eq!(
            second_column_start_position(line, line.chars().count() / 2),
            Some(15)
        );

        let line = "first column        second column";
        assert_eq!(
            second_column_start_position(line, line.chars().count() / 2),
            Some(20)
        );

        let line = "first column        second column";
        assert_eq!(
            second_column_start_position(line, line.chars().count() / 2),
            Some(20)
        );

        let line = "first column        second column      third column";
        assert_eq!(
            second_column_start_position(line, line.chars().count() / 2),
            None
        );

        let line = "   first column        second column";
        assert_eq!(
            second_column_start_position(line, line.chars().count() / 2),
            Some(23)
        );

        // TODO
        // let line = "first column        second column   ";
        // assert_eq!(second_column_start_position(line, line.chars().count()/2), Some(20));
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
}
