use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    inner_lines: Vec<String>,
    pub context_lines: Vec<String>,
}

impl Patch {
    pub fn extract(
        text: &[&str],
        index: usize,
        stop: &Regex,
        start: usize,
        end: usize,
        up: bool,
        nb_line: usize,
    ) -> Self {
        let mut content_lines = Vec::new();
        let mut context_lines = Vec::new();

        // compute real start and end from the base
        let (_content, _before, left, right) = complete(text[index], start, end).unwrap();

        if up {
            for i in (index.saturating_sub(6)..=index).rev() {
                if let Some((content, before, _, _)) = complete(text[i], left, right) {
                    let should_stop = stop.is_match(&content) || stop.is_match(&before);

                    content_lines.insert(0, content);
                    context_lines.insert(0, before);

                    if should_stop {
                        break;
                    }
                }
            }
        } else {
            for line in text
                .iter()
                .take((index + nb_line).min(text.len()))
                .skip(index)
            {
                if let Some((content, before, _, _)) = complete(line, left, right) {
                    let should_stop = stop.is_match(&content) || stop.is_match(&before);

                    content_lines.push(content);
                    context_lines.push(before);

                    if should_stop {
                        break;
                    }
                }
            }
        }

        Patch {
            inner_lines: content_lines,
            context_lines,
        }
    }

    pub fn lines(&self) -> Vec<String> {
        self.inner_lines
            .iter()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect()
    }
}

fn complete(line: &str, start: usize, end: usize) -> Option<(String, String, usize, usize)> {
    let chars: Vec<char> = line.chars().collect();
    let left = left_complete(line, start, end)?;
    let right = right_complete(line, left, end)?;
    Some((
        String::from_iter(&chars[left..=right]),
        String::from_iter(&chars[..left]),
        left,
        right,
    ))
}

fn left_complete(line: &str, mut start: usize, mut end: usize) -> Option<usize> {
    if end >= line.chars().count() {
        end = line.chars().count().saturating_sub(1);
    }

    if start >= line.chars().count() || start > end {
        return None;
    }

    let chars: Vec<char> = line.chars().collect();

    while start > 0 && !chars[start..=end].starts_with(&[' ', ' ']) {
        start -= 1;
    }

    if chars[start..=end].starts_with(&[' ', ' ']) {
        start += 2;
    } else if chars[start..=end].starts_with(&[' ']) {
        start += 1;
    }

    Some(start)
}

pub fn right_complete(line: &str, start: usize, mut end: usize) -> Option<usize> {
    if end >= line.chars().count() {
        end = line.chars().count().saturating_sub(1);
    }

    if start >= line.chars().count() || start > end {
        return None;
    }

    let chars: Vec<char> = line.chars().collect();

    while end < line.chars().count() - 1 && !chars[start..=end].ends_with(&[' ', ' ']) {
        end += 1;
    }

    if chars[start..=end].ends_with(&[' ', ' ']) {
        end -= 2;
    } else if chars[start..=end].ends_with(&[' ']) {
        end -= 1;
    }

    Some(end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract() {
        let stop = Regex::new(r"(?i)(titulaire)").unwrap();

        let text = vec![
            "bla bla",
            "titulaire : M OU MME MATISSE HENRI",
            "51 RUE BERNARD ROY",
            "44100 NANTES",
        ];

        let patch = Patch::extract(&text, 3, &stop, 0, 11, true, 3);

        let result = vec![
            "titulaire : M OU MME MATISSE HENRI",
            "51 RUE BERNARD ROY",
            "44100 NANTES",
        ];

        assert_eq!(patch.inner_lines, result);

        let text = vec![
            "bla bla    M OU MME MATISSE HENRI",
            "bla bla    51 RUE BERNARD ROY",
            "bla bla    44100 NANTES",
        ];

        let result = vec![
            "M OU MME MATISSE HENRI",
            "51 RUE BERNARD ROY",
            "44100 NANTES",
        ];

        let patch = Patch::extract(&text, 2, &stop, 10, 17, true, 3);

        assert_eq!(patch.inner_lines, result);

        let text = vec![
            "M OU MME MATISSE HENRI",
            "51 RUE BERNARD ROY",
            "44166 NANTES SUR LA LONGUE VILLE",
        ];

        let patch = Patch::extract(&text, 2, &stop, 0, 31, true, 3);

        let result = vec![
            "M OU MME MATISSE HENRI",
            "51 RUE BERNARD ROY",
            "44166 NANTES SUR LA LONGUE VILLE",
        ];

        assert_eq!(patch.inner_lines, result);

        let text = vec!["M HENRI", "51 RUE BERNARD ROY, 44100 NANTES"];

        let patch = Patch::extract(&text, 1, &stop, 20, 31, true, 3);

        let result = vec!["M HENRI", "51 RUE BERNARD ROY, 44100 NANTES"];

        assert_eq!(patch.inner_lines, result);
    }

    #[test]
    fn test_left_complete() {
        assert_eq!(left_complete("🦀123  6", 0, 0), Some(0));
        assert_eq!(left_complete("0123  678 0🦀  4", 10, 11), Some(6));
    }

    #[test]
    fn test_right_complete() {
        assert_eq!(right_complete("0123  🦀", 6, 6), Some(6));
        assert_eq!(right_complete("0123  🦀78 01  4", 6, 7), Some(11));
    }

    #[test]
    fn test_complete() {
        assert_eq!(
            complete("0  34 🦀78 01  4", 7, 7),
            Some(("34 🦀78 01".to_string(), "0  ".to_string(), 3, 11))
        );
    }
}
