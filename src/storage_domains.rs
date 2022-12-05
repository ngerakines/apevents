use crate::{
    error::ApEventsError,
    state::MyStateHandle,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

#[derive(Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Domain {
    domain: String,
    action: i32
}

impl FromRow<'_, PgRow> for Domain {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            domain: row.try_get("domain")?,
            action: row.try_get("action")?,
        })
    }
}

impl Domain {
    pub fn is_allowed(&self) -> bool {
        true
    }
}

pub async fn create_domain(app_state: &MyStateHandle, domain: String) -> Result<Domain, ApEventsError> {
    sqlx::query(
        "INSERT INTO domains (domain) VALUES ($1) ON CONFLICT ON CONSTRAINT domains_pkey DO NOTHING",
    )
    .bind(&domain)
    .execute(&app_state.pool)
    .await
    .map_err(ApEventsError::conv)?;

    sqlx::query_as("SELECT * FROM domains WHERE domain = $1")
    .bind(&domain)
    .fetch_one(&app_state.pool)
    .await
    .map_err(ApEventsError::conv)
}

pub async fn list_allowed_domains(app_state: &MyStateHandle) -> Result<Vec<String>, ApEventsError> {
    let results: Vec<Domain> = sqlx::query_as("SELECT * FROM domains WHERE action = 0")
            .fetch_all(&app_state.pool)
            .await.map_err(ApEventsError::conv)?;
    Ok(results.into_iter().map(|d| d.domain ).collect())
}
