use actix_webfinger::Webfinger;
use log::{debug};
use reqwest::Url;

use crate::ap;
use crate::error::ApEventsError;
use crate::objects::actor::EventActor;
use crate::state::MyStateHandle;
use crate::util::fetch_object_http;
use crate::webfinger::webfinger_discover;

pub async fn actor_maybe(
    app_state: &MyStateHandle,
    local_ap_id: String,
    remote_actor_ref: String,
) -> Result<EventActor, ApEventsError> {
    // TODO: Lock on remote_ap_id

    /*
    1. Is the actor local?
       1. Is the local actor valid?
          1. Return an EventActor
    2. Do we have a copy of the actor?
       1. Return an EventActor
    3. Can we fetch the actor?
       1. Is the actor reference a valid URL?
          1. Can we request the actor?
    */

    let found_follower_res: Result<EventActor, sqlx::Error> =
        sqlx::query_as("SELECT * FROM actors WHERE ap_id = $1")
            .bind(&local_ap_id)
            .fetch_one(&app_state.pool)
            .await;
    let found_follower = found_follower_res.unwrap();

    let found_actor_res: Result<Option<EventActor>, sqlx::Error> =
        sqlx::query_as("SELECT * FROM actors WHERE $1 = ANY (resources)")
            .bind(&remote_actor_ref)
            .fetch_optional(&app_state.pool)
            .await;

    if let Err(err) = found_actor_res {
        return Err(err.into());
    }

    let found_actor = found_actor_res.unwrap();

    if let Some(_) = found_actor {
        return Ok(found_actor.unwrap());
    }

    let webfinger_res: Webfinger = webfinger_discover(remote_actor_ref).await?;

    if webfinger_res.activitypub().is_none() || webfinger_res.activitypub().unwrap().href.is_none() {
        return Err(ApEventsError::new("unable to get activitypub id from resource".to_string()));
    }
    let remote_ap_id = webfinger_res.activitypub().unwrap().href.as_ref().unwrap();

    let remote_ap_id_url = Url::parse(&remote_ap_id)?;
    debug!("remote_ap_id_url {}", &remote_ap_id);

    let public_key_id = format!("{}#main-key", local_ap_id);
    let it: ap::actor::Actor = fetch_object_http(
        &remote_ap_id_url,
        public_key_id,
        found_follower.private_key.unwrap(),
    )
    .await?;

    debug!("it {:?}", &it);

    Ok(it.try_into()?)
}
