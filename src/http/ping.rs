use actix_web::http::header::CONTENT_TYPE;
use actix_web::HttpResponse;
use actix_web::{get, Responder};

#[get("/ping")]
pub async fn ping() -> impl Responder {
    let mut response = match std::env::current_dir() {
        Ok(path_buff) => {
            if path_buff.join("maintenance").exists() {
                HttpResponse::NotFound()
            } else {
                HttpResponse::Ok()
            }
        }

        Err(_) => HttpResponse::InternalServerError(),
    };

    response
        .insert_header((CONTENT_TYPE, "application/json"))
        .body("{}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::dev::ServiceResponse;
    use actix_web::{test, App};
    use serial_test::serial;
    use std::fs;

    async fn call_ping() -> ServiceResponse {
        let app = test::init_service(App::new().service(ping)).await;
        let req = test::TestRequest::get().uri("/ping").to_request();
        test::call_service(&app, req).await
    }

    fn maintenance_path() -> std::path::PathBuf {
        std::env::current_dir().unwrap().join("maintenance")
    }

    #[actix_web::test]
    #[serial(servers)]
    async fn test_ping_ok_when_no_maintenance_file() {
        if maintenance_path().exists() {
            fs::remove_file(maintenance_path()).unwrap();
        }

        let resp = call_ping().await;

        assert!(resp.status().is_success());
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    #[serial(servers)]
    async fn test_ping_not_found_when_maintenance_file_exists() {
        fs::write(maintenance_path(), "").unwrap();

        let resp = call_ping().await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);

        fs::remove_file(maintenance_path()).unwrap();
    }
}
