use std::env;

use actix_web::{http::header, web, web::Data, App, HttpResponse, HttpServer, Responder};

mod state;
mod webfinger;

use crate::state::MyState;
use crate::webfinger::*;

async fn handle_index() -> impl Responder {
    HttpResponse::Ok()
        .append_header(header::ContentType(mime::TEXT_PLAIN))
        .body("Hello World!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let domain: String =
        env::var("APEVENTS_DOMAIN").unwrap_or_else(|_| format!("localhost:{port}"));

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(MyState {
                domain: domain.to_owned(),
                https: false,
            }))
            .service(actix_webfinger::resource::<MyResolver>())
            .route("/", web::get().to(handle_index))
    })
    .bind(format!("0.0.0.0:{port}"))?
    .run()
    .await
}
