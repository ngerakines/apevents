use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

use crate::ap::ids::ObjectIdError;

#[derive(Debug, Error)]
pub enum ApEventsError {
    #[error("{0}")]
    Generic(String),

    #[error("error")]
    SQLError,

    #[error("an unexpected error has occured")]
    Unknown,

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    ParseError(#[from] url::ParseError),

    #[error("an unexpected error has occured")]
    ObjectIdError(#[from] ObjectIdError),

    #[error("an unexpected error has occured")]
    JsonError(#[from] serde_json::Error),

    #[error("an unexpected error has occured")]
    ActivityPubFederation(#[from] activitypub_federation::Error),

    #[error("an unexpected error has occured")]
    SignatureVerificationError(
        #[from] http_signature_normalization_actix::digest::middleware::VerifyError,
    ),

    #[error("an unexpected error has occured")]
    InstanceSettingsBuilderError(#[from] activitypub_federation::InstanceSettingsBuilderError),

    #[error("an unexpected error has occured")]
    ClientRequestMiddlewareError(#[from] reqwest_middleware::Error),

    #[error("an unexpected error has occured")]
    ClientRequestError(#[from] reqwest::Error),

    #[error("actor not found: {0}")]
    ActorNotFound(String, #[source] anyhow::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}

impl ApEventsError {
    pub fn new(message: String) -> Self {
        ApEventsError::Generic(message)
    }

    pub fn name(&self) -> String {
        match self {
            Self::ActorNotFound(_, _) => "Actor Not Found".to_string(),
            Self::Generic(_) => "Generic".to_string(),
            Self::Unknown => "Unknown".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}

impl ResponseError for ApEventsError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::ActorNotFound(_, _) => StatusCode::NOT_FOUND,
            Self::Generic(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
            error: self.name(),
        };
        HttpResponse::build(status_code).json(error_response)
    }
}
