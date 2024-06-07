use actix_web::{http::header::ContentType, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::io::Read;
use ureq::Error;

use crate::{datamatrix::fetch_datamatrix, file_utils::bytes_to_img, twoddoc::parse};

#[derive(Deserialize)]
struct RequestedFile {
    url: String,
}

#[derive(Deserialize, Serialize)]
pub struct UpstreamError {
    pub upstream_body: String,
    pub upstream_status_code: u16,
}

#[post("/analyze")]
pub async fn analyze(requested_file: web::Json<RequestedFile>) -> impl Responder {
    match ureq::get(&requested_file.url).call() {
        Ok(resp) => handle_response(resp),
        Err(error) => handle_error(error),
    }
}

fn handle_error(error: Error) -> HttpResponse {
    match error {
        Error::Status(code, response) => {
            let upstream_body = match response.into_string() {
                Ok(body) => body,
                Err(_) => { "unreadable upstream error".to_string() }
            };

            let error = UpstreamError {
                upstream_body,
                upstream_status_code: code,
            };

            HttpResponse::BadGateway().json(error)
        },
        Error::Transport(_) => HttpResponse::InternalServerError().body("Error"),
    }
}

fn handle_response(resp: ureq::Response) -> HttpResponse {
    let len: usize = resp.header("Content-Length").unwrap().parse().unwrap();

    let mut bytes: Vec<u8> = Vec::with_capacity(len);
    resp.into_reader()
        .take(10_000_000)
        .read_to_end(&mut bytes)
        .unwrap();

    let img = bytes_to_img(bytes);

    let datamatrix = fetch_datamatrix(img);

    if let Some(datamatrix) = datamatrix {
        let ddoc = parse(&datamatrix).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(ddoc)
    } else {
        println!("No datamatrix found");
        HttpResponse::UnprocessableEntity().body("No datamatrix found")
    }
}
