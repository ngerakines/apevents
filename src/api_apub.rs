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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowersCollectionPage {
    #[serde(rename = "id")]
    pub ap_id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub total_items: u32,
    pub part_of: String,
    pub first: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    pub items: Vec<String>,
}

#[derive(Deserialize)]
pub struct Pagination {
    pub page: Option<i32>,
}

pub async fn handle_instance_get_event_actor_followers(
    name: web::Path<String>,
    app_state: web::Data<MyStateHandle>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, ApEventsError> {
    // TODO: Validate signatures

    let request_url = format!("{}/actor/{}", app_state.external_base, name);
    let url = Url::parse(&request_url)?;
    let user = ObjectId::<EventActor>::new(url)
        .dereference_local(&app_state)
        .await?;

    let ap_id = user.followers_url()?.to_string();

    let total: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM follow_activities WHERE followee_ap_id = $1")
            .bind(&request_url)
            .fetch_one(&app_state.pool)
            .await?;

    let first: Option<String> = match total.0 {
        0 => None,
        _ => Some(format!("{}/?page=1", ap_id)),
    };

    if pagination.page.is_none() {
        return Ok(HttpResponse::Ok()
            .content_type(APUB_JSON_CONTENT_TYPE)
            .json(WithContext::new(
                FollowersCollection {
                    ap_id,
                    kind: "OrderedCollection".to_string(),
                    total_items: total.0 as u32,
                    first,
                },
                vec![Value::from_str(
                    "\"https://www.w3.org/ns/activitystreams\"",
                )?],
            )));
    }

    let page = pagination.page.unwrap();

    if page < 0 {
        return Err(ApEventsError::Generic(
            "invalid query string parameter: page".to_string(),
        ));
    }

    let offset = (100 * page) - 100;

    let items: Vec<(String,)> = sqlx::query_as(
        "SELECT followee_ap_id FROM follow_activities WHERE followee_ap_id = $1 ORDER BY created_at ASC LIMIT 100 OFFSET $2",
    )
    .bind(&request_url)
    .bind(offset)
    .fetch_all(&app_state.pool)
    .await?;

    let prev = match page {
        1 => None,
        _ => Some(format!("{}?page={}", ap_id, page - 1)),
    };

    let next = match items.len() {
        100 => Some(format!("{}?page={}", ap_id, page + 1)),
        _ => None,
    };

    Ok(HttpResponse::Ok()
        .content_type(APUB_JSON_CONTENT_TYPE)
        .json(WithContext::new(
            FollowersCollectionPage {
                ap_id: format!("{}?page={}", ap_id, page),
                kind: "OrderedCollection".to_string(),
                total_items: total.0 as u32,
                part_of: ap_id,
                first,
                next,
                prev,
                items: items.iter().map(|i| i.0.clone()).collect(),
            },
            vec![Value::from_str(
                "\"https://www.w3.org/ns/activitystreams\"",
            )?],
        )))
}

pub async fn handle_instance_get_event_actor_following(
    name: web::Path<String>,
    app_state: web::Data<MyStateHandle>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, ApEventsError> {
    // TODO: Validate signatures

    let request_url = format!("{}/actor/{}", app_state.external_base, name);
    let url = Url::parse(&request_url)?;
    let user = ObjectId::<EventActor>::new(url)
        .dereference_local(&app_state)
        .await?;

    let ap_id = user.following_url()?.to_string();

    let total: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM follow_activities WHERE follower_ap_id = $1")
            .bind(&request_url)
            .fetch_one(&app_state.pool)
            .await?;

    let first: Option<String> = match total.0 {
        0 => None,
        _ => Some(format!("{}/?page=1", ap_id)),
    };

    if pagination.page.is_none() {
        return Ok(HttpResponse::Ok()
            .content_type(APUB_JSON_CONTENT_TYPE)
            .json(WithContext::new(
                FollowersCollection {
                    ap_id,
                    kind: "OrderedCollection".to_string(),
                    total_items: total.0 as u32,
                    first,
                },
                vec![Value::from_str(
                    "\"https://www.w3.org/ns/activitystreams\"",
                )?],
            )));
    }

    let page = pagination.page.unwrap();

    if page < 0 {
        return Err(ApEventsError::Generic(
            "invalid query string parameter: page".to_string(),
        ));
    }

    let offset = (100 * page) - 100;

    let items: Vec<(String,)> = sqlx::query_as(
        "SELECT followee_ap_id FROM follow_activities WHERE follower_ap_id = $1 ORDER BY created_at ASC LIMIT 100 OFFSET $2",
    )
    .bind(&request_url)
    .bind(offset)
    .fetch_all(&app_state.pool)
    .await?;

    let prev = match page {
        1 => None,
        _ => Some(format!("{}?page={}", ap_id, page - 1)),
    };

    let next = match items.len() {
        100 => Some(format!("{}?page={}", ap_id, page + 1)),
        _ => None,
    };

    Ok(HttpResponse::Ok()
        .content_type(APUB_JSON_CONTENT_TYPE)
        .json(WithContext::new(
            FollowersCollectionPage {
                ap_id: format!("{}?page={}", ap_id, page),
                kind: "OrderedCollection".to_string(),
                total_items: total.0 as u32,
                part_of: ap_id,
                first,
                next,
                prev,
                items: items.iter().map(|i| i.0.clone()).collect(),
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
