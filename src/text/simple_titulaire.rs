use regex::Regex;

use super::patch::{right_complete, Patch};

pub fn find_simple_titulaire(text: &str, nb_line: usize) -> Option<Vec<String>> {
    let titulaire = Regex::new(r"(?i)(titulaire)").unwrap();
    let stop = Regex::new(r"(?i)(domiciliation|cadre réservé|identification)").unwrap();
    let lines: Vec<&str> = text.lines().collect();

    let result = if let Some((index, m)) = lines.iter().enumerate().find_map(|(i, line)| {
        if titulaire.is_match(line) {
            Some((i, titulaire.find(line).unwrap()))
        } else {
            None
        }
    }) {
        let start = m.start();
        let end = m.end();

        let patch = Patch::extract(&lines, index, &stop, start, end - 1, false, nb_line);
        clean(patch.lines())
    } else {
        None
    };

    if result.is_some() {
        return result;
    }

    // last chance, try for one line by civilite
    let civilite =
        Regex::new(r"(?i)(^|\s)(m|monsieur|mr|mademoiselle|ml|mle|mlle|melle|madame|mme)\.?\s")
            .unwrap();

    if let Some(index) = lines.iter().position(|x| civilite.is_match(x)) {
        let m = civilite.find(lines[index]).unwrap();
        let start = m.start();

        return right_complete(lines[index], start, m.end() - 1)
            .map(|end| vec![lines[index][start..=end].trim().to_string()]);
    }

    None
}

fn clean(lines: Vec<String>) -> Option<Vec<String>> {
    let titulaire = Regex::new(r"(?i)(titulaire)").unwrap();
    let headers =
        Regex::new(r"(?i)(titulaire|domiciliation|cadre réservé|identification|numero de)")
            .unwrap();

    let vec: Vec<String> = lines
        .into_iter()
        .map(|line| {
            if titulaire.is_match(&line) && line.contains(':') {
                line.split(':').nth(1).unwrap().to_string()
            } else {
                line
            }
        })
        .filter(|line| !headers.is_match(line))
        .filter(|line| !line.is_empty())
        .map(|line| {
            if line.contains(':') {
                line.split(':').nth(1).unwrap().to_string()
            } else {
                line
            }
        })
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    if vec.is_empty() {
        None
    } else {
        Some(vec)
    }
}
