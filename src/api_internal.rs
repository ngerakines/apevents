use crate::state::MyState;
use actix_web::http::header;
use actix_web::{web, HttpResponse, Responder};
use tracing::info;

pub async fn handle_internal_create_user(app_state: web::Data<MyState>) -> impl Responder {
    let name = app_state.create_user().await;
    if let Err(err) = name {
        info!("error {:?}", err);
        return HttpResponse::InternalServerError().finish();
    }
    HttpResponse::Ok()
        .append_header(header::ContentType(mime::TEXT_PLAIN))
        .body(name.unwrap())
}
