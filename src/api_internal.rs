use crate::error::ApEventsError;
use crate::state::MyStateHandle;
use activitypub_federation::core::signatures::generate_actor_keypair;
use actix_web::http::header;
use actix_web::{web, HttpResponse};
use serde::Deserialize;

pub async fn handle_internal_create_user(
    app_state: web::Data<MyStateHandle>,
) -> Result<HttpResponse, ApEventsError> {
    let name = petname::Petnames::default().generate_one(3, "-");
    let object_id = format!("{}/actor/{}", app_state.external_base, name);
    let inbox_id = format!("{}/actor/{}/inbox", app_state.external_base, name);
    let actor_ref = format!("@{}@{}", name, app_state.domain);

    let keypair = generate_actor_keypair()?;

    let insert_result = sqlx::query("insert into actors (ap_id, actor_ref, is_local, inbox_id, public_key, private_key) values ($1, $2, true, $3, $4, $5)")
        .bind(object_id)
        .bind(actor_ref)
        .bind(inbox_id)
        .bind(keypair.public_key)
        .bind(keypair.private_key)
        .execute(&app_state.pool)
        .await;

    if let Err(_) = insert_result {
        return Ok(HttpResponse::InternalServerError().finish());
    }

    Ok(HttpResponse::Ok()
        .append_header(header::ContentType(mime::TEXT_PLAIN))
        .body(name))
}

// #[derive(Deserialize)]
// struct FollowRequest {
//     follower: String,
//     followee: String
// }

// pub async fn handle_internal_follow_remote(
//     app_state: web::Data<MyStateHandle>,
//     follow_request: web::Json<FollowRequest>,
// ) -> Result<HttpResponse, ApEventsError> {
//     let name = petname::Petnames::default().generate_one(3, "-");
//     let object_id = format!("{}/actor/{}", app_state.external_base, name);
//     let inbox_id = format!("{}/actor/{}/inbox", app_state.external_base, name);
//     let actor_ref = format!("@{}@{}", name, app_state.domain);
//     let keypair = generate_actor_keypair()?;
//     let insert_result = sqlx::query("insert into actors (ap_id, actor_ref, is_local, inbox_id, public_key, private_key) values ($1, $2, true, $3, $4, $5)")
//         .bind(object_id)
//         .bind(actor_ref)
//         .bind(inbox_id)
//         .bind(keypair.public_key)
//         .bind(keypair.private_key)
//         .execute(&app_state.pool)
//         .await;

//     if let Err(_) = insert_result {
//         return Ok(HttpResponse::InternalServerError().finish());
//     }

//     Ok(HttpResponse::Ok()
//         .append_header(header::ContentType(mime::TEXT_PLAIN))
//         .body(name))
// }
