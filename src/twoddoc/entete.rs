use chrono::NaiveDate;

#[derive(Debug, PartialEq)]
pub struct Entete {
    pub autorite_certification: String,
    pub identifiant_du_certificat: String,
    pub date_emission: Option<NaiveDate>,
    pub date_creation_signature: NaiveDate,
    pub type_document_id: String,
    pub perimetre: Option<String>,
    pub emetteur: Option<String>,
}

impl From<(&str, &str, Option<NaiveDate>, NaiveDate, &str)> for Entete {
    fn from(data: (&str, &str, Option<NaiveDate>, NaiveDate, &str)) -> Self {
        Entete {
            autorite_certification: data.0.to_string(),
            identifiant_du_certificat: data.1.to_string(),
            date_emission: data.2,
            date_creation_signature: data.3,
            type_document_id: data.4.to_string(),
            perimetre: None,
            emetteur: None,
        }
    }
}

impl From<(&str, &str, Option<NaiveDate>, NaiveDate, &str, &str)> for Entete {
    fn from(data: (&str, &str, Option<NaiveDate>, NaiveDate, &str, &str)) -> Self {
        Entete {
            autorite_certification: data.0.to_string(),
            identifiant_du_certificat: data.1.to_string(),
            date_emission: data.2,
            date_creation_signature: data.3,
            type_document_id: data.4.to_string(),
            perimetre: Some(data.5.to_string()),
            emetteur: None,
        }
    }
}

impl From<(&str, &str, Option<NaiveDate>, NaiveDate, &str, &str, &str)> for Entete {
    fn from(data: (&str, &str, Option<NaiveDate>, NaiveDate, &str, &str, &str)) -> Self {
        Entete {
            autorite_certification: data.0.to_string(),
            identifiant_du_certificat: data.1.to_string(),
            date_emission: data.2,
            date_creation_signature: data.3,
            type_document_id: data.4.to_string(),
            perimetre: Some(data.5.to_string()),
            emetteur: Some(data.6.to_string()),
        }
    }
}
