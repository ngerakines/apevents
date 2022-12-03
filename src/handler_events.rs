use actix_web::{
    web::{Data, Path},
    HttpResponse, Result,
};
use askama_actix::{Template, TemplateToResponse};

use crate::{objects::actor::EventActor, state::MyStateHandle};

use crate::error::ApEventsError;

#[derive(Template)]
#[template(path = "event.html")]
struct EventTemplate<'a> {
    display_name: &'a str,
    ap_id: &'a str,
    actor_ref: &'a str,
    summary: &'a str,
    when: &'a str,
    location: &'a str,
    follower_count: u32,
    attendee_count: u32,
    hidden_attendee_count: u32,
}

struct EventElementTemplate(String, String);

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate<'a> {
    display_name: &'a str,
    events: Vec<EventElementTemplate>,
}

pub async fn handle_home(app_state: Data<MyStateHandle>) -> Result<HttpResponse, ApEventsError> {
    let found_actors: Result<Vec<EventActor>, sqlx::Error> = sqlx::query_as("SELECT * FROM actors")
        .fetch_all(&app_state.pool)
        .await;

    if let Err(_) = found_actors {
        return Ok(HttpResponse::NotFound().finish());
    }

    Ok(HomeTemplate {
        display_name: "A cool event",
        events: found_actors
            .unwrap()
            .iter()
            .map(|x| EventElementTemplate(x.ap_id.to_string(), x.actor_ref.clone()))
            .collect(),
    }
    .to_response())
}

pub async fn handle_event(
    info: Path<String>,
    app_state: Data<MyStateHandle>,
) -> Result<HttpResponse, ApEventsError> {
    let actor_ap_id = format!("{}/actor/{}", app_state.external_base, info);

    let found_actor_res: Result<EventActor, sqlx::Error> =
        sqlx::query_as("SELECT * FROM actors WHERE ap_id = $1")
            .bind(actor_ap_id)
            .fetch_one(&app_state.pool)
            .await;

    if let Err(_) = found_actor_res {
        return Ok(HttpResponse::NotFound().finish());
    }
    let found_actor = found_actor_res.unwrap();

    Ok(EventTemplate {
        display_name: "A cool event",
        ap_id: &found_actor.ap_id.to_string(),
        actor_ref: &found_actor.actor_ref,
        summary: "Welcome!",
        when: "11/1/2022 at 7:00 PM eastern",
        location: "The bar, 555 nowhere, dayton, oh 45419",
        follower_count: 25,
        attendee_count: 3,
        hidden_attendee_count: 0,
    }
    .to_response())
}
