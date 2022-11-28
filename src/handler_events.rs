use actix_web::{HttpResponse, Result, web::{Data, Path}};
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

pub async fn handle_event(info: Path<String>, app_state: Data<MyStateHandle>) -> Result<HttpResponse, ApEventsError> {
    let actor_ap_id = format!("{}/actor/{}", app_state.external_base, info);
    let actor_ref = format!("@{}@{}", info, app_state.domain);

    let found_actor_res :Result<EventActor, sqlx::Error> = sqlx::query_as("SELECT * FROM actors WHERE ap_id = $1")
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
        actor_ref: &actor_ref.to_string(),
        summary: "Welcome!",
        when: "11/1/2022 at 7:00 PM eastern",
        location: "The bar, 555 nowhere, dayton, oh 45419",
        follower_count: 25,
        attendee_count: 3,
        hidden_attendee_count: 0,
     }.to_response())
}
