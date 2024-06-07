use super::analyze;
use actix_web::{middleware, App, HttpServer};

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(analyze::analyze)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
