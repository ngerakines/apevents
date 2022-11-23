use std::env;

use actix_web::{http::header, web, App, HttpResponse, HttpServer, Responder};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod error;
mod state;
mod webfinger;

use actix_webfinger::WebfingerGuard;

use crate::state::state_factory;
use crate::webfinger::handle_webfinger;

async fn handle_index() -> impl Responder {
    HttpResponse::Ok()
        .append_header(header::ContentType(mime::TEXT_PLAIN))
        .body("Hello World!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let listen_address: String =
        env::var("LISTEN_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
    let listen_port: String = env::var("LISTEN_PORT")
        .unwrap_or_else(|_| env::var("PORT").unwrap_or_else(|_| "8080".to_string()));

    let addrs = format!("{listen_address}:{listen_port}");

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .data_factory(state_factory)
            .service(
                actix_web::web::resource("/.well-known/webfinger")
                    .guard(WebfingerGuard)
                    .route(web::get().to(handle_webfinger)),
            )
            .route("/", web::get().to(handle_index))
    })
    .bind(addrs)?
    .run()
    .await
}
