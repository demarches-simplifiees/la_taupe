use std::io::Read;
use url::Url;
use x509_cert::{
    der::{Decode, DecodePem},
    Certificate,
};

use crate::twoddoc::trust_service::trust_service;

pub fn certificate(autorite_certification: &str, identifiant_du_certificat: &str) -> Certificate {
    if autorite_certification == "FR00" {
        let cert_path = "tests/fixtures/certificates/certificate_FR00_00.pem";

        let certificate_bytes = std::fs::read(cert_path).unwrap();

        Certificate::from_pem(&certificate_bytes[..]).unwrap()
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

    let resp = ureq::get(url.as_str()).call().unwrap();

    let len: usize = resp.header("Content-Length").unwrap().parse().unwrap();

    let mut bytes: Vec<u8> = Vec::with_capacity(len);
    resp.into_reader()
        .take(10_000_000)
        .read_to_end(&mut bytes)
        .unwrap();

    Certificate::from_der(&bytes[..]).unwrap()
}
