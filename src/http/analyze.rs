use actix_web::{http::header::ContentType, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::io::Read;
use ureq::Error;

use crate::analysis::Analysis;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

#[derive(Deserialize)]
struct RequestedFile {
    url: String,
}

#[derive(Deserialize, Serialize)]
pub struct AnalysisError {
    pub upstream_body: Option<String>,
    pub upstream_status_code: Option<u16>,
    pub body: Option<String>,
}

#[post("/analyze")]
pub async fn analyze(requested_file: web::Json<RequestedFile>) -> impl Responder {
    match ureq::AgentBuilder::new()
        .try_proxy_from_env(true)
        .build()
        .get(&requested_file.url)
        .call()
    {
        Ok(resp) => handle_response(resp),
        Err(error) => handle_error(error),
    }
}

fn handle_error(error: Error) -> HttpResponse {
    match error {
        Error::Status(code, response) => {
            let upstream_body = match response.into_string() {
                Ok(body) => Some(body),
                Err(_) => Some("unreadable upstream error".to_string()),
            };

            let error = AnalysisError {
                upstream_body,
                upstream_status_code: Some(code),
                body: None,
            };

            HttpResponse::BadGateway().json(error)
        }
        Error::Transport(_) => HttpResponse::InternalServerError().body("Error"),
    }
}

fn handle_response(resp: ureq::Response) -> HttpResponse {
    let len: usize = match resp.header("Content-Length") {
        Some(len) => len.parse().unwrap(),
        None => MAX_FILE_SIZE,
    };

    let mut bytes: Vec<u8> = Vec::with_capacity(len);

    let size = resp
        .into_reader()
        .take((MAX_FILE_SIZE + 1).try_into().unwrap())
        .read_to_end(&mut bytes)
        .unwrap();

    if size > MAX_FILE_SIZE {
        return HttpResponse::UnprocessableEntity()
            .content_type(ContentType::json())
            .json(AnalysisError {
                upstream_status_code: None,
                upstream_body: None,
                body: Some("File too big".to_string()),
            });
    }

    match Analysis::try_into(bytes) {
        Ok(analysis) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(analysis),
        Err(error_msg) => HttpResponse::UnprocessableEntity()
            .content_type(ContentType::json())
            .json(AnalysisError {
                upstream_status_code: None,
                upstream_body: None,
                body: Some(error_msg),
            }),
    }
}
