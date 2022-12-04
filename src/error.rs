use actix_web::{http::header, HttpResponse, ResponseError};
use anyhow::anyhow;
use std::fmt::{Display, Formatter};

/// Necessary because of this issue: https://github.com/actix/actix-web/issues/1711
#[derive(Debug)]
pub struct ApEventsError(anyhow::Error);

impl ApEventsError {
    pub fn new(message: String) -> Self {
        Self(anyhow!(message))
    }
}

impl ResponseError for ApEventsError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError()
            .append_header(header::ContentType(mime::TEXT_PLAIN))
            .body(format!("{:?}", self.0))
    }
}

impl Display for ApEventsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl<T> From<T> for ApEventsError
where
    T: Into<anyhow::Error>,
{
    fn from(t: T) -> Self {
        ApEventsError(t.into())
    }
}

impl ApEventsError {
    pub fn conv<T>(error: T) -> Self
    where
        T: Into<anyhow::Error>,
    {
        ApEventsError(error.into())
    }
}

