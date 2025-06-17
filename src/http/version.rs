use actix_web::{get, HttpResponse, Result};
use serde_json::json;
use std::process::Command;

#[get("/version")]
pub async fn version() -> Result<HttpResponse> {
    let la_taupe_version = env!("GIT_HASH");
    let tesseract_version = tesseract_version();

    Ok(HttpResponse::Ok().json(json!({
        "la_taupe": la_taupe_version,
        "tesseract": tesseract_version
    })))
}

fn tesseract_version() -> String {
    match Command::new("tesseract").arg("--version").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let _stderr = String::from_utf8_lossy(&output.stderr);
            // tesseract écrit sa version sur stderr généralement
            if !stdout.is_empty() {
                stdout.trim().to_string()
            } else {
                "unknown".to_string()
            }
        }
        Err(_) => "not available".to_string(),
    }
}
