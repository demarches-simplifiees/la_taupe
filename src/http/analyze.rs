use actix_web::{http::header::ContentType, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::io::Read;
use ureq::{http::Response, Agent, Body};

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
    let proxy = ureq::Proxy::try_from_env();

    let agent: Agent = Agent::config_builder()
        .http_status_as_error(false)
        .proxy(proxy)
        .build()
        .into();

    let response = agent.get(&requested_file.url).call().unwrap();

    if response.status().is_success() {
        handle_response(response)
    } else {
        handle_error(response)
    }
}

fn handle_error(mut resp: Response<Body>) -> HttpResponse {
    let upstream_body = Some(
        resp.body_mut()
            .read_to_string()
            .unwrap_or_else(|_| "unreadable upstream error".to_string()),
    );
    let upstream_status_code = Some(resp.status().into());

    if resp.status().is_server_error() {
        let error = AnalysisError {
            upstream_body,
            upstream_status_code,
            body: Some("upstream server error".to_string()),
        };

        HttpResponse::BadGateway().json(error)
    } else {
        let error = AnalysisError {
            upstream_body,
            upstream_status_code,
            body: Some("upstream client error".to_string()),
        };

        HttpResponse::InternalServerError().json(error)
    }
}

fn handle_response(mut resp: Response<Body>) -> HttpResponse {
    let len: usize = if let Some(len) = resp.headers().get("Content-Length") {
        len.to_str()
            .unwrap_or("")
            .parse::<usize>()
            .unwrap_or(MAX_FILE_SIZE)
    } else {
        MAX_FILE_SIZE
    };

    let mut bytes: Vec<u8> = Vec::with_capacity(len);

    let size = resp
        .body_mut()
        .as_reader()
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

    match Analysis::try_from(bytes) {
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
