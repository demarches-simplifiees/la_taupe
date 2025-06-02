use std::io::Read;
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

    log::info!("Fetching certificate from {}", url);

    let mut resp = ureq::get(url.as_str()).call().unwrap();

    let len: usize = resp
        .headers()
        .get("Content-Length")
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();

    let mut bytes: Vec<u8> = Vec::with_capacity(len);

    resp.body_mut().as_reader().read_to_end(&mut bytes).unwrap();

    Certificate::from_der(&bytes[..]).unwrap()
}
