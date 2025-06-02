mod helpers;
pub use helpers::*;
use la_taupe::{analysis::Analysis, http::analyze::AnalysisError};
use serde_json::json;
use ureq::Agent;

use static_init::dynamic;

#[dynamic(drop)]
static mut TAUPE_AND_NODE: TaupeAndNode = TaupeAndNode::start();

#[test]
fn nominal_case() {
    let mut response = ureq::post("http://localhost:8080/analyze")
        .send_json(json!({
            "url": "http://localhost:3333/justificatif_de_domicile.png"
        }))
        .unwrap();

    let analysis: Analysis = response.body_mut().read_json().unwrap();

    assert_eq!(
        analysis.ddoc.unwrap().entete.autorite_certification,
        "FR00".to_string()
    );
}

fn agent() -> Agent {
    Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .into()
}

#[test]
fn upstream_error() {
    let mut response = agent()
        .post("http://localhost:8080/analyze")
        .send_json(json!({
            "url": "http://localhost:3333/500"
        }))
        .unwrap();

    assert_eq!(response.status(), 502);

    let analysis: AnalysisError = response.body_mut().read_json().unwrap();

    assert_eq!(analysis.upstream_status_code.unwrap(), 500);
    assert_eq!(analysis.upstream_body.unwrap(), "KO: 500".to_string());
}

#[test]
fn missing_datamatrix() {
    let mut response = ureq::post("http://localhost:8080/analyze")
        .send_json(json!({
            "url": "http://localhost:3333/la_taupe.png"
        }))
        .unwrap();

    let analysis: Analysis = response.body_mut().read_json().unwrap();

    assert!(analysis.ddoc.is_none());
}

#[test]
fn file_too_big() {
    let mut response = agent()
        .post("http://localhost:8080/analyze")
        .send_json(json!({
            "url": "http://localhost:3333/too_big"
        }))
        .unwrap();

    assert_eq!(response.status(), 422);

    let analysis: AnalysisError = response.body_mut().read_json().unwrap();

    assert_eq!(analysis.body.unwrap(), "File too big".to_string());
}

#[test]
fn unhandled_format() {
    let mut response = agent()
        .post("http://localhost:8080/analyze")
        .send_json(json!({
            "url": "http://localhost:3333/text.txt"
        }))
        .unwrap();

    assert_eq!(response.status(), 422);

    let analysis: AnalysisError = response.body_mut().read_json().unwrap();

    assert_eq!(
        analysis.body.unwrap(),
        "Unsupported file type: text/plain".to_string()
    );
}
