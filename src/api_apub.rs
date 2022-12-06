use crate::{
    error::ApEventsError,
    objects::actor::{EventActor, PersonAcceptedActivities},
    state::MyStateHandle,
};
use activitypub_federation::{
    core::{inbox::receive_activity, object_id::ObjectId},
    data::Data,
    deser::context::WithContext,
    traits::ApubObject,
    APUB_JSON_CONTENT_TYPE,
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{ops::Deref, str::FromStr, vec};
use url::Url;

pub async fn handle_wellknown_host_meta(
    app_state: web::Data<MyStateHandle>,
) -> Result<HttpResponse, ApEventsError> {
    Ok(HttpResponse::Ok()
    .content_type("application/xrd+xml")
    .body(format!(r#"<?xml version="1.0" encoding="UTF-8"?><XRD xmlns="http://docs.oasis-open.org/ns/xri/xrd-1.0"><Link rel="lrdd" template="{}/.well-known/webfinger?resource={{uri}}"/></XRD>"#, app_state.external_base)))
}

pub async fn handle_instance_get_event_actor(
    name: web::Path<String>,
    app_state: web::Data<MyStateHandle>,
) -> Result<HttpResponse, ApEventsError> {
    // TODO: Validate signatures

    let request_url = format!("{}/actor/{}", app_state.external_base, name);
    let url = Url::parse(&request_url)?;
    let user = ObjectId::<EventActor>::new(url)
        .dereference_local(&app_state)
        .await?;

    Ok(HttpResponse::Ok()
        .content_type(APUB_JSON_CONTENT_TYPE)
        .json(WithContext::new(
            user.into_apub(&app_state).await?,
            vec![
                Value::from_str("\"https://www.w3.org/ns/activitystreams\"")?,
                Value::from_str("\"https://w3id.org/security/v1\"")?,
            ],
        )))
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowersCollection {
    #[serde(rename = "id")]
    pub ap_id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub total_items: u32,
}

pub async fn handle_instance_get_event_actor_followers(
    name: web::Path<String>,
    app_state: web::Data<MyStateHandle>,
) -> Result<HttpResponse, ApEventsError> {
    // TODO: Validate signatures

    let request_url = format!("{}/actor/{}", app_state.external_base, name);
    let url = Url::parse(&request_url)?;
    let user = ObjectId::<EventActor>::new(url)
        .dereference_local(&app_state)
        .await?;

    Ok(HttpResponse::Ok()
        .content_type(APUB_JSON_CONTENT_TYPE)
        .json(WithContext::new(
            FollowersCollection {
                ap_id: user.followers_url()?.to_string(),
                kind: "OrderedCollection".to_string(),
                total_items: 0,
            },
            vec![Value::from_str(
                "\"https://www.w3.org/ns/activitystreams\"",
            )?],
        )))
}

pub async fn handle_instance_get_event_actor_following(
    name: web::Path<String>,
    app_state: web::Data<MyStateHandle>,
) -> Result<HttpResponse, ApEventsError> {
    // TODO: Validate signatures

    let request_url = format!("{}/actor/{}", app_state.external_base, name);
    let url = Url::parse(&request_url)?;
    let user = ObjectId::<EventActor>::new(url)
        .dereference_local(&app_state)
        .await?;

    Ok(HttpResponse::Ok()
        .content_type(APUB_JSON_CONTENT_TYPE)
        .json(WithContext::new(
            FollowersCollection {
                ap_id: user.followers_url()?.to_string(),
                kind: "OrderedCollection".to_string(),
                total_items: 0,
            },
            vec![Value::from_str(
                "\"https://www.w3.org/ns/activitystreams\"",
            )?],
        )))
}

pub async fn handle_instance_post_event_actor_inbox(
    request: HttpRequest,
    payload: String,
    app_state: web::Data<MyStateHandle>,
) -> Result<HttpResponse, ApEventsError> {
    let data: MyStateHandle = app_state.into_inner().deref().clone();
    let activity = serde_json::from_str(&payload)?;
    receive_activity::<WithContext<PersonAcceptedActivities>, EventActor, MyStateHandle>(
        request,
        activity,
        &data.clone().local_instance,
        &Data::new(data),
    )
    .await
}
