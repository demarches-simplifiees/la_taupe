mod helpers;
pub use helpers::*;
use la_taupe::{http::analyze::UpstreamError, twoddoc::ddoc::Ddoc};
use ureq::Error;

use static_init::dynamic;

#[dynamic(drop)]
static mut TAUPE_AND_NODE: TaupeAndNode = TaupeAndNode::start();

#[test]
fn nominal_case() {
    let response = ureq::post("http://localhost:8080/analyze")
        .send_json(ureq::json!({
            "url": "http://localhost:3333/justificatif_de_domicile.png"
        }))
        .unwrap();

    let ddoc: Ddoc = response.into_json().unwrap();

    assert_eq!(
        ddoc.entete.autorite_certification,
        "FR00".to_string()
    );
}

#[test]
fn upstream_error() {
    let error = ureq::post("http://localhost:8080/analyze")
        .send_json(ureq::json!({
            "url": "http://localhost:3333/500"
        }))
        .err()
        .unwrap();

    match error {
        Error::Status(502, response) => {
            let error: UpstreamError = response.into_json().unwrap();
            assert_eq!(error.upstream_status_code, 500);
            assert_eq!(error.upstream_body, "KO: 500".to_string());
        },
        _ => panic!("Expected a 500 error"),
    }
}

#[test]
fn missing_datamatrix() {
}
