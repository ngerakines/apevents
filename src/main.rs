extern crate env_logger;

use std::env;

use actix_web::{http::header, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use api_apub::{handle_instance_post_event_actor_inbox, handle_wellknown_host_meta};
use api_internal::handle_internal_follow_remote;
use http_signature_normalization_actix::prelude::VerifyDigest;
use sha2::{Digest, Sha256};

mod activities;
mod ap;
mod api_apub;
mod api_internal;
mod api_nodeinfo;
mod error;
mod fed;
mod handler_events;
mod instance;
mod objects;
mod state;
mod storage_actor;
mod util;
mod webfinger;

use actix_files as fs;
use actix_webfinger::WebfingerGuard;
use util::HeaderStart;

use crate::api_apub::handle_instance_get_event_actor;
use crate::api_internal::handle_internal_create_user;
use crate::api_nodeinfo::{
    handle_instance_info_v1, handle_instance_peers, handle_nodeinfo_20, handle_wellknown_nodeinfo,
};
use crate::handler_events::{handle_event, handle_home};
use crate::state::state_factory;
use crate::webfinger::handle_webfinger;

async fn handle_index() -> impl Responder {
    HttpResponse::Ok()
        .append_header(header::ContentType(mime::TEXT_PLAIN))
        .body("Hello World!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let listen_address: String =
        env::var("LISTEN_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
    let listen_port: String = env::var("LISTEN_PORT")
        .unwrap_or_else(|_| env::var("PORT").unwrap_or_else(|_| "8080".to_string()));

    let addrs = format!("{listen_address}:{listen_port}");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%a %r %s %T '%{User-Agent}i'").log_target("apevents::web"))
            .service(fs::Files::new("/static", "./static/"))
            .data_factory(state_factory)
            .service(
                web::scope("")
                    .guard(HeaderStart("accept", "text/html"))
                    .route("/", web::get().to(handle_home))
                    .route("/actor/{name}", web::get().to(handle_event))
                    .route("/@{name}", web::get().to(handle_event)),
            )
            .service(
                actix_web::web::resource("/.well-known/webfinger")
                    .guard(WebfingerGuard)
                    .route(web::get().to(handle_webfinger)),
            )
            .route(
                "/.well-known/host-meta",
                web::get().to(handle_wellknown_host_meta),
            )
            .route(
                "/.well-known/nodeinfo",
                web::get().to(handle_wellknown_nodeinfo),
            )
            .route("/nodeinfo/2.0", web::get().to(handle_nodeinfo_20))
            .route("/api/v1/instance", web::get().to(handle_instance_info_v1))
            .route(
                "/api/v1/instance/peers",
                web::get().to(handle_instance_peers),
            )
            .route("/", web::get().to(handle_index))
            .route(
                "/actor/{name}",
                web::get().to(handle_instance_get_event_actor),
            )
            .route(
                "/internal/api/user",
                web::post().to(handle_internal_create_user),
            )
            .route(
                "/internal/api/follow",
                web::post().to(handle_internal_follow_remote),
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
