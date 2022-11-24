use crate::{error::ApEventsError, objects::person::MyUser, state::MyState};
use activitypub_federation::{
    core::object_id::ObjectId, deser::context::WithContext, traits::ApubObject,
    APUB_JSON_CONTENT_TYPE,
};
use actix_web::{web, HttpRequest, HttpResponse};
use url::Url;

pub async fn handle_instance_get_user(
    request: HttpRequest,
    app_state: web::Data<MyState>,
) -> Result<HttpResponse, ApEventsError> {
    let request_url = format!("{}{}", app_state.external_base, &request.uri().to_string());
    let url = Url::parse(&request_url)?;
    let user = ObjectId::<MyUser>::new(url)
        .dereference_local(&app_state)
        .await;
    if let Err(_) = user {
        return Ok(HttpResponse::NotFound()
            .content_type(APUB_JSON_CONTENT_TYPE)
            .finish());
    }
    Ok(HttpResponse::Ok()
        .content_type(APUB_JSON_CONTENT_TYPE)
        .json(WithContext::new_default(
            user.unwrap().into_apub(&app_state).await?,
        )))
}
