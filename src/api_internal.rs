use std::collections::HashMap;

use activitypub_federation::{
    core::signatures::generate_actor_keypair, deser::context::WithContext, traits::ApubObject,
    APUB_JSON_CONTENT_TYPE,
};
use actix_web::{http::header, web, HttpResponse};
use serde::Deserialize;

use crate::ap::actor::{Actor, PublicKey};
use crate::error::ApEventsError;
use crate::objects::actor::EventActor;
use crate::state::MyStateHandle;
use crate::storage_actor::create_actor;

pub async fn handle_internal_create_user(
    app_state: web::Data<MyStateHandle>,
) -> Result<HttpResponse, ApEventsError> {
    let name = petname::Petnames::default().generate_one(3, "-");
    let object_id = format!("{}/actor/{}", app_state.external_base, name);

    let keypair = generate_actor_keypair().map_err(|_| ApEventsError::Unknown)?;

    // pub async fn create_actor(app_state: &MyStateHandle, actor: Actor) -> Result<EventActor, ApEventsError> {
    create_actor(
        &app_state,
        Actor {
            ap_id: object_id.clone(),
            kind: "Person".to_string(),

            following: Some(format!("{}/actor/{}/inbox", app_state.external_base, name)),
            followers: Some(format!("{}/actor/{}/inbox", app_state.external_base, name)),
            inbox: Some(format!("{}/actor/{}/inbox", app_state.external_base, name)),
            outbox: None,
            featured: None,

            featured_tags: None,

            name: name.clone(),
            summary: None,
            preferred_username: Some(name.clone()),

            url: Some(format!("{}/@{}", app_state.external_base, name.clone())),

            discoverable: None,
            published: None,

            public_key: Some(PublicKey {
                ap_id: format!("{}/actor/{}#main-key", app_state.external_base, name),
                owner: object_id,
                public_key_pem: keypair.public_key,
            }),

            attachments: vec![],

            endpoints: HashMap::from([(
                "sharedInbox".to_string(),
                format!("{}/inbox", app_state.external_base),
            )]),

            icon: None,
            image: None,
        },
        Some(keypair.private_key),
    )
    .await?;

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
    let found_actor: EventActor = sqlx::query_as("SELECT * FROM actors WHERE ap_id = $1")
        .bind(follow_request.follower.clone())
        .fetch_one(&app_state.pool)
        .await?;

    found_actor
        .follow(follow_request.followee.clone(), &app_state)
        .await?;

    Ok(HttpResponse::Ok()
        .content_type(APUB_JSON_CONTENT_TYPE)
        .json(WithContext::new_default(
            found_actor.into_apub(&app_state).await?,
        )))
}
