use reqwest::blocking::Client;
use url::Url;
use x509_cert::{
    der::{Decode, DecodePem},
    Certificate,
};

use crate::twoddoc::trust_service::trust_service;

static TEST_CERTIFICATE: &[u8] =
    include_bytes!("../../tests/fixtures/certificates/certificate_FR00_00.pem");

pub fn certificate(autorite_certification: &str, identifiant_du_certificat: &str) -> Certificate {
    if autorite_certification == "FR00" {
        Certificate::from_pem(TEST_CERTIFICATE).unwrap()
    } else {
        fetch_certificate(autorite_certification, identifiant_du_certificat)
    }
}

fn fetch_certificate(autorite_certification: &str, identifiant_du_certificat: &str) -> Certificate {
    let service = trust_service(autorite_certification);

    let mut url: Url = service.information_url;
    url.query_pairs_mut()
        .append_pair("name", identifiant_du_certificat);

    log::trace!("Fetching certificate from {}", url);

    let client = Client::new();
    let resp = client.get(url.as_str()).send().unwrap();

    if !resp.status().is_success() {
        panic!("Failed to fetch certificate: HTTP {}", resp.status());
    }

    let bytes = resp.bytes().unwrap().to_vec();
    Certificate::from_der(&bytes[..]).unwrap()
}
