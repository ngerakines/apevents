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
    request: HttpRequest,
    app_state: web::Data<MyStateHandle>,
) -> Result<HttpResponse, ApEventsError> {
    // TODO: Validate signatures

    let request_url = format!("{}{}", app_state.external_base, &request.uri().to_string());
    let url = Url::parse(&request_url)?;
    let user = ObjectId::<EventActor>::new(url)
        .dereference_local(&app_state)
        .await;
    if user.is_err() {
        return Ok(HttpResponse::NotFound()
            .content_type(APUB_JSON_CONTENT_TYPE)
            .finish());
    }
    Ok(HttpResponse::Ok()
        .content_type(APUB_JSON_CONTENT_TYPE)
        .json(WithContext::new(
            user.unwrap().into_apub(&app_state).await?,
            vec![
                Value::from_str("\"https://www.w3.org/ns/activitystreams\"")?,
                Value::from_str("\"https://w3id.org/security/v1\"")?,
            ],
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
