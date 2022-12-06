use std::time::Duration;

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_webfinger::Webfinger;
use log::info;
use reqwest::{Client, Url};
use reqwest_middleware::ClientWithMiddleware;
use serde::de::DeserializeOwned;
use serde_derive::Deserialize;

use crate::{error::ApEventsError, objects::actor::EventActor, state::MyStateHandle};

#[derive(Clone, Debug, Deserialize)]
pub struct WebfingerQuery {
    resource: String,
}

pub async fn handle_webfinger(
    app_state: web::Data<MyStateHandle>,
    _req: HttpRequest,
    query: web::Query<WebfingerQuery>,
) -> impl Responder {
    let mut query_resource: Option<&str> = None;
    if query.resource.starts_with("acct:") {
        if !query
            .resource
            .ends_with(format!("@{}", app_state.domain).as_str())
        {
            info!(
                "resource not in domain '{}' '{}'",
                query.resource, app_state.domain
            );
            // Bail if the resource does not end with "@<domain>"
            return HttpResponse::NotFound().finish();
        }
        query_resource = query.resource.strip_prefix("acct:");
    } else if query.resource.starts_with("https://") {
        // Bail if the resource does not start with with "<external_base>/"
        // This is because the external_base includes the protocol ("https://"), domain, and port.
        if !query
            .resource
            .starts_with(format!("{}/", app_state.external_base).as_str())
        {
            return HttpResponse::NotFound().finish();
        }
        query_resource = Some(query.resource.as_str());
    }

    if query_resource.is_none() {
        return HttpResponse::NotFound().finish();
    }

    let found_actor_res: Result<Option<EventActor>, sqlx::Error> =
        sqlx::query_as("SELECT * FROM actors WHERE $1 = ANY (resources)")
            .bind(query_resource.unwrap())
            .fetch_optional(&app_state.pool)
            .await;

    if found_actor_res.is_err() {
        return HttpResponse::NotFound().finish();
    }

    let found_actor = found_actor_res.unwrap();

    if found_actor.is_none() {
        return HttpResponse::NotFound().finish();
    }

    let fa = found_actor.unwrap();
    let actor_ref_parts: Vec<&str> = fa.actor_ref.split('@').collect();
    let name = actor_ref_parts[0];

    let mut svc = Webfinger::new(&format!("acct:{}", fa.actor_ref));
    svc.add_alias(&format!("{}/@{}", app_state.external_base, &name));
    svc.add_profile(&format!("{}/@{}", app_state.external_base, &name));
    svc.add_activitypub(&fa.ap_id.to_string());
    svc.respond()
}

pub async fn webfinger_discover<Kind: DeserializeOwned>(
    resource: String,
) -> Result<Kind, ApEventsError> {
    let mut domain: Option<String> = None;

    if resource.starts_with("https://") {
        let parsed_resource = Url::parse(resource.clone().as_str())?;
        domain = parsed_resource.domain().map(|value| value.to_string())
    } else if resource.starts_with("acc:") {
        let trimmed: &str = resource.trim_start_matches('@');
        domain = (!trimmed.is_empty()).then(|| trimmed.to_string());
    }
    if domain.is_none() {
        return Err(ApEventsError::new(
            "unable to parse domain from resource".to_string(),
        ));
    }

    let client: ClientWithMiddleware = Client::default().into();
    let request_timeout = Duration::from_secs(10);

    let res = client
        .get(format!("https://{}/.well-known/webfinger", domain.unwrap()))
        .query(&[("resource", resource)])
        .timeout(request_timeout)
        .header("Accept", "application/jrd+json")
        .send()
        .await?;

    res.json().await.map_err(|err| err.into())
}
