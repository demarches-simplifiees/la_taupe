use regex::Regex;

use super::patch::Patch;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AddrType {
    Titulaire,
    Domiciliation,
    Unknown,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Addr {
    pub inner_lines: Vec<String>,
    pub addr_type: AddrType,
}

impl Addr {
    pub fn lines(&self) -> Vec<String> {
        let header = Regex::new(r"(?i)(titulaire|intitulé|identit[e|é] bancaire)").unwrap();
        let intitule = Regex::new(r"(?i)(intitulé du compte)").unwrap();

        self.inner_lines
            .clone()
            .into_iter()
            .map(|line| {
                if line.contains(':') {
                    line.split(':').nth(1).unwrap().to_string()
                } else {
                    line
                }
            })
            .map(|line| intitule.replace_all(&line, "").to_string())
            .filter(|line| !header.is_match(line))
            .map(|line| line.trim().to_string())
            .collect()
    }
}

pub fn find_titulaire_addr(text: &str) -> Option<Addr> {
    let lines: Vec<&str> = text.split("\n").collect();
    let code_postal = Regex::new(r"(^| )\d{5} ([[:alpha:]]+ ?)+").unwrap();
    let patch_upper_limit =
        Regex::new(r"(?i)(titulaire|intitulé|domiciliation|cadre réservé)").unwrap();

    let patches = lines
        .clone()
        .into_iter()
        .enumerate()
        .flat_map(|(index, line)| {
            code_postal.find(line).map(|m| {
                Patch::extract(
                    &lines,
                    index,
                    &patch_upper_limit,
                    m.start(),
                    m.end() - 1,
                    true,
                    3,
                )
            })
        });

    let addresses = patches.map(|p| {
        let addr_type = addr_type_from_text(&p);
        Addr {
            inner_lines: p.lines(),
            addr_type,
        }
    });

    let addr = addresses
        .clone()
        .filter(|addr| addr.addr_type == AddrType::Titulaire)
        .collect::<Vec<Addr>>()
        .first()
        .cloned();

    if let Some(addr) = addr {
        return Some(addr);
    }
    let addr = addresses
        .filter(|addr| addr.addr_type == AddrType::Unknown)
        .collect::<Vec<Addr>>()
        .first()
        .cloned();

    addr
}

fn addr_type_from_text(patch: &Patch) -> AddrType {
    let titulaire = Regex::new(r"(?i)(titulaire|intitulé)").unwrap();
    let domiciliation = Regex::new(r"(?i)(domiciliation|cadre réservé)").unwrap();

    let text = patch.lines().join(" ");

    if titulaire.is_match(&text) {
        return AddrType::Titulaire;
    } else if domiciliation.is_match(&text) {
        return AddrType::Domiciliation;
    }

    let context = patch.context_lines.join(" ");

    if titulaire.is_match(&context) {
        return AddrType::Titulaire;
    } else if domiciliation.is_match(&context) {
        return AddrType::Domiciliation;
    }
    AddrType::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vec_to_string(v: Vec<&str>) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    fn test_file(path: &str, titulaire: Vec<&str>) {
        let layout_text = std::fs::read_to_string(path).unwrap();
        let titulaire = Some(vec_to_string(titulaire));
        let addrs = find_titulaire_addr(&layout_text);
        assert_eq!(addrs.map(|a| a.lines()), titulaire);
    }

    #[test]
    fn addr_banque_populaire() {
        let path = "tests/fixtures/rib/banque_populaire.txt";
        let titulaire = vec![
            "M OU MME MATISSE HENRI",
            "51 RUE BERNARD ROY",
            "44100 NANTES",
        ];

        test_file(path, titulaire);
    }

    #[test]
    fn addr_banque_postale() {
        let path = "tests/fixtures/rib/banque_postale.txt";
        let titulaire = vec![
            "MR MATISSE HENRI",
            "243 RUE DES GRIVES",
            "44240 LA CHAPELLE SUR ERDRE",
        ];

        test_file(path, titulaire);
    }

    #[test]
    fn addr_credit_mutuel_2() {
        let path = "tests/fixtures/rib/credit_mutuel_2.txt";
        let titulaire = vec![
            "M OU MME MATISSE HENRI",
            "54 RUE DE L HERONNIERE",
            "44000 NANTES",
        ];

        test_file(path, titulaire);
    }
}
