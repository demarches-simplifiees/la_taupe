mod helpers;
pub use helpers::*;
use la_taupe::{analysis::Analysis, http::analyze::UpstreamError};
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

    let analysis: Analysis = response.into_json().unwrap();

    assert_eq!(
        analysis.ddoc.unwrap().entete.autorite_certification,
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
    let response = ureq::post("http://localhost:8080/analyze")
        .send_json(ureq::json!({
            "url": "http://localhost:3333/la_taupe.png"
        }))
        .unwrap();

    let analysis: Analysis = response.into_json().unwrap();

    assert!(analysis.ddoc.is_none());
}
