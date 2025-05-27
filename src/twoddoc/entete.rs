use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Entete {
    pub autorite_certification: String,
    pub identifiant_du_certificat: String,
    pub date_emission: Option<NaiveDateTime>,
    pub date_creation_signature: NaiveDateTime,
    pub type_document_id: String,
    pub type_document: String,
    pub perimetre: Option<String>,
    pub emetteur: Option<String>,
}

impl From<(&str, &str, Option<NaiveDateTime>, NaiveDateTime, &str)> for Entete {
    fn from(data: (&str, &str, Option<NaiveDateTime>, NaiveDateTime, &str)) -> Self {
        Entete {
            autorite_certification: data.0.to_string(),
            identifiant_du_certificat: data.1.to_string(),
            date_emission: data.2,
            date_creation_signature: data.3,
            type_document_id: data.4.to_string(),
            type_document: type_de_document_id_to_libelle(data.4),
            perimetre: None,
            emetteur: None,
        }
    }
}

impl From<(&str, &str, Option<NaiveDateTime>, NaiveDateTime, &str, &str)> for Entete {
    fn from(data: (&str, &str, Option<NaiveDateTime>, NaiveDateTime, &str, &str)) -> Self {
        Entete {
            autorite_certification: data.0.to_string(),
            identifiant_du_certificat: data.1.to_string(),
            date_emission: data.2,
            date_creation_signature: data.3,
            type_document_id: data.4.to_string(),
            type_document: type_de_document_id_to_libelle(data.4),
            perimetre: Some(data.5.to_string()),
            emetteur: None,
        }
    }
}

impl
    From<(
        &str,
        &str,
        Option<NaiveDateTime>,
        NaiveDateTime,
        &str,
        &str,
        &str,
    )> for Entete
{
    fn from(
        data: (
            &str,
            &str,
            Option<NaiveDateTime>,
            NaiveDateTime,
            &str,
            &str,
            &str,
        ),
    ) -> Self {
        Entete {
            autorite_certification: data.0.to_string(),
            identifiant_du_certificat: data.1.to_string(),
            date_emission: data.2,
            date_creation_signature: data.3,
            type_document_id: data.4.to_string(),
            type_document: type_de_document_id_to_libelle(data.4),
            perimetre: Some(data.5.to_string()),
            emetteur: Some(data.6.to_string()),
        }
    }
}

fn type_de_document_id_to_libelle(id: &str) -> String {
    let libelle = match id {
        "00" | "01" | "02" => "Justificatif de domicile",
        "03" | "05" | "11" => "Documents bancaires",
        "09" | "19" | "20" | "21" => "Justificatif fiscal",
        "04" | "18" | "06" => "Justificatif de ressources",
        "10" | "15" => "Justificatif d’emploi",
        "07" | "08" | "13" => "Justificatif d’identité",
        "A0" | "A7" | "14" => "Justificatif de véhicule",
        "A8" => "Certificat d’immatriculation",
        "A1" | "AA" | "AB" => "Justificatif permis de conduire",
        "B0" | "B1" => "Justificatif académique",
        "A4" | "AE" => "Justificatif médical",
        "A2" => "Justificatif de santé",
        "A3" | "A5" | "A6" | "A9" | "AC" => "Justificatif d’activité",
        "12" => "Justificatif juridique/judiciaire",
        "22" | "C1" | "C2" | "C3" | "C4" | "C5" | "C6" | "C7" | "C8" => "Autorisations douanières",
        "B2" => "Résultats des tests virologiques",
        "L1" => "Attestation Vaccinale",
        "16" | "17" => "Justificatif d’Asile",
        "C9" => "Caducée Infirmier",
        _ => "unknown",
    };

    libelle.to_string()
}
