use std::env;

use super::analyze;
use actix_web::{middleware, App, HttpServer};
use std::net::{SocketAddr, ToSocketAddrs};

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(analyze::analyze)
    })
    .bind(binding_address())?
    .run()
    .await
}

pub fn binding_address() -> SocketAddr {
    let address = env::var("LA_TAUPE_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
        .to_string();
    address.to_socket_addrs().unwrap().next().unwrap()
}
