use std::collections::HashMap;

// fetch from https://www.ecb.europa.eu/stats/financial_corporations/list_of_financial_institutions/html/monthly_list-MID.en.html
// then iconv -f UTF-16 -t UTF-8 fi_mrr_csv_250630.csv | grep "^FR" | cut -f1,4 > src/riad_bank_name.csv
const RIAD_CSV: &str = include_str!("./riad_bank_name.csv");

pub struct IbanToBankName {
    data: HashMap<String, String>,
}

impl IbanToBankName {
    pub fn new() -> Self {
        let mut data = HashMap::new();

        for line in RIAD_CSV.lines() {
            // Skip header
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() >= 2 {
                let riad_code = fields[0].to_string();
                let name = fields[1].to_string();
                data.insert(riad_code, name);
            }
        }

        Self { data }
    }

    pub fn bank_name(&self, iban: &str) -> Option<String> {
        let iban_without_space = iban.replace(" ", "");
        let country_code = iban_without_space.chars().take(2).collect::<String>();
        let bank_code = iban_without_space
            .chars()
            .skip(4)
            .take(5)
            .collect::<String>();
        let riad_code = format!("{}{}", country_code, bank_code);

        self.data.get(&riad_code).cloned()
    }
}

impl Default for IbanToBankName {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_bic_and_name() {
        let fi_extract = IbanToBankName::new();

        let result = fi_extract.bank_name("FR0042529ANDSTUFF");
        assert_eq!(result, Some("Edmond de Rothschild (France)".to_string()));

        // Test avec un RIAD_CODE inexistant
        let result = fi_extract.bank_name("NONEXISTENT");
        assert_eq!(result, None);
    }
}
