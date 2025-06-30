mod helpers;
pub use helpers::*;
use la_taupe::{analysis::Analysis, http::analyze::AnalysisError};
use reqwest::blocking::Client;
use serde_json::json;

use static_init::dynamic;

#[dynamic(drop)]
static mut TAUPE_AND_NODE: TaupeAndNode = TaupeAndNode::start();

#[test]
fn nominal_case() {
    let response = Client::new()
        .post("http://localhost:8080/analyze")
        .json(&json!({
            "url": "http://localhost:3333/justificatif_de_domicile.png",
            "hint": { "type": "2ddoc" },
        }))
        .send()
        .unwrap();

    if let Analysis::Ddoc { ddoc } = response.json().unwrap() {
        assert_eq!(
            ddoc.unwrap().entete.autorite_certification,
            "FR00".to_string()
        );
    } else {
        panic!("Expected Analysis::OnlyDdoc");
    }
}

#[test]
fn upstream_error() {
    let response = Client::new()
        .post("http://localhost:8080/analyze")
        .json(&json!({
            "url": "http://localhost:3333/500",
            "hint": { "type": "2ddoc" },
        }))
        .send()
        .unwrap();

    assert_eq!(response.status().as_u16(), 502);

    let analysis: AnalysisError = response.json().unwrap();

    assert_eq!(analysis.upstream_status_code.unwrap(), 500);
    assert_eq!(analysis.upstream_body.unwrap(), "KO: 500".to_string());
}

#[test]
fn missing_datamatrix() {
    let response = Client::new()
        .post("http://localhost:8080/analyze")
        .json(&json!({
            "url": "http://localhost:3333/la_taupe.png",
            "hint": { "type": "2ddoc" },
        }))
        .send()
        .unwrap();

    if let Analysis::Ddoc { ddoc } = response.json().unwrap() {
        assert!(ddoc.is_none());
    } else {
        panic!("Expected Analysis::OnlyDdoc");
    }
}

#[test]
fn file_too_big() {
    let response = Client::new()
        .post("http://localhost:8080/analyze")
        .json(&json!({
            "url": "http://localhost:3333/too_big",
            "hint": { "type": "2ddoc" },
        }))
        .send()
        .unwrap();

    assert_eq!(response.status().as_u16(), 422);

    let analysis: AnalysisError = response.json().unwrap();

    assert_eq!(analysis.body.unwrap(), "File too big".to_string());
}

#[test]
fn unhandled_format() {
    let response = Client::new()
        .post("http://localhost:8080/analyze")
        .json(&json!({
            "url": "http://localhost:3333/text.txt",
            "hint": { "type": "2ddoc" },
        }))
        .send()
        .unwrap();

    assert_eq!(response.status().as_u16(), 422);

    let analysis: AnalysisError = response.json().unwrap();

    assert_eq!(
        analysis.body.unwrap(),
        "Unsupported file type: text/plain".to_string()
    );
}

// #[test]
// TODO: make it work on ci
#[allow(dead_code)]
fn empty_pdf() {
    let response = Client::new()
        .post("http://localhost:8080/analyze")
        .json(&json!({
            "url": "http://localhost:3333/la_taupe.pdf",
            "hint": { "type": "rib" },
        }))
        .send()
        .unwrap();

    assert_eq!(response.status().as_u16(), 422);

    let analysis: AnalysisError = response.json().unwrap();

    assert_eq!(
        analysis.body.unwrap(),
        "Failed to extract text from PDF".to_string()
    );
}
