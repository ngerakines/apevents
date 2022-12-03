use crate::error::ApEventsError;
use crate::fed::actor_maybe;
use crate::objects::actor::EventActor;
use crate::state::MyStateHandle;
use activitypub_federation::core::signatures::generate_actor_keypair;
use activitypub_federation::deser::context::WithContext;
use activitypub_federation::traits::ApubObject;
use activitypub_federation::APUB_JSON_CONTENT_TYPE;
use actix_web::http::header;
use actix_web::{web, HttpResponse};
use serde::Deserialize;

pub async fn handle_internal_create_user(
    app_state: web::Data<MyStateHandle>,
) -> Result<HttpResponse, ApEventsError> {
    let name = petname::Petnames::default().generate_one(3, "-");
    let object_id = format!("{}/actor/{}", app_state.external_base, name);
    let inbox_id = format!("{}/actor/{}/inbox", app_state.external_base, name);
    let actor_ref = format!("{}@{}", name, app_state.domain);

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

#[derive(Deserialize)]
pub struct FollowRequest {
    follower: String,
    followee: String,
}

pub async fn handle_internal_follow_remote(
    app_state: web::Data<MyStateHandle>,
    follow_request: web::Json<FollowRequest>,
) -> Result<HttpResponse, ApEventsError> {
    // let found_actor: EventActor = sqlx::query_as("SELECT * FROM actors WHERE ap_id = $1")
    // .bind(&follow_request.followee)
    // .fetch_one(&app_state.pool)
    // .await?;

    let found_remote_actor = actor_maybe(
        &app_state,
        follow_request.follower.clone(),
        follow_request.followee.clone(),
    )
    .await?;

    Ok(HttpResponse::Ok()
        .content_type(APUB_JSON_CONTENT_TYPE)
        .json(WithContext::new_default(
            found_remote_actor.into_apub(&app_state).await?,
        )))

    // found_actor
    //     .follow(&beta.local_user(), &alpha)
    //     .await?;

    // let name = petname::Petnames::default().generate_one(3, "-");
    // let object_id = format!("{}/actor/{}", app_state.external_base, name);
    // let inbox_id = format!("{}/actor/{}/inbox", app_state.external_base, name);
    // let actor_ref = format!("@{}@{}", name, app_state.domain);
    // let keypair = generate_actor_keypair()?;
    // let insert_result = sqlx::query("insert into actors (ap_id, actor_ref, is_local, inbox_id, public_key, private_key) values ($1, $2, true, $3, $4, $5)")
    //     .bind(object_id)
    //     .bind(actor_ref)
    //     .bind(inbox_id)
    //     .bind(keypair.public_key)
    //     .bind(keypair.private_key)
    //     .execute(&app_state.pool)
    //     .await;

    // if let Err(_) = insert_result {
    //     return Ok(HttpResponse::InternalServerError().finish());
    // }

    // Ok(HttpResponse::Ok()
    //     .append_header(header::ContentType(mime::TEXT_PLAIN))
    //     .body("".to_string()))
}
