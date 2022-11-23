use std::env;

use actix_web::{http::header, web, App, HttpResponse, HttpServer, Responder};

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
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    HttpServer::new(move || {
        App::new()
            .data_factory(state_factory)
            .service(
                actix_web::web::resource("/.well-known/webfinger")
                    .guard(WebfingerGuard)
                    .route(web::get().to(handle_webfinger)),
            )
            .route("/", web::get().to(handle_index))
    })
    .bind(format!("0.0.0.0:{port}"))?
    .run()
    .await
}
