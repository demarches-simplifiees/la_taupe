use std::env;

use super::{analyze, ping, version};
use actix_web::{middleware::Logger, App, HttpServer};
use env_logger::Env;
use std::io::Write;
use std::net::{SocketAddr, ToSocketAddrs};

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::new(r#"{"timestamp":"%t","method":"%r","status":%s,"response_time":%D,"remote_addr":"%a","user_agent":"%{User-Agent}i","remote_file":"%{X-Remote-File}i"}"#))
            .service(analyze::analyze)
            .service(ping::ping)
            .service(version::version)
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
