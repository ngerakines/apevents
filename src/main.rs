use std::env;

use actix_web::{http::header, web, App, HttpResponse, HttpServer, Responder};
use api_apub::handle_instance_post_event_actor_inbox;
use http_signature_normalization_actix::prelude::VerifyDigest;
use sha2::{Digest, Sha256};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod activities;
mod api_apub;
mod api_internal;
mod error;
mod instance;
mod objects;
mod state;
mod util;
mod webfinger;

use actix_webfinger::WebfingerGuard;

use crate::api_apub::handle_instance_get_event_actor;
use crate::api_internal::handle_internal_create_user;
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
            .route("/actor/{name}", web::get().to(handle_instance_get_event_actor))
            .route(
                "/internal/api/user",
                web::post().to(handle_internal_create_user),
            )
            .service(
                web::scope("")
                    .wrap(VerifyDigest::new(Sha256::new()))
                    .route(
                        "/inbox",
                        web::post().to(handle_instance_post_event_actor_inbox),
                    )
                    .route(
                        "/actor/{name}/inbox",
                        web::post().to(handle_instance_post_event_actor_inbox),
                    ),
            )
    })
    .bind(addrs)?
    .run()
    .await
}
