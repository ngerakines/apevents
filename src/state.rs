use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

use crate::error::ApEventsError;

#[derive(Clone, Debug)]
pub struct MyState {
    pub domain: String,
    pub external_base: String,
    pub database: String,

    pub pool: Pool<Postgres>,
}

pub async fn state_factory() -> Result<MyState, ApEventsError> {
    let domain: String = env::var("DOMAIN").unwrap_or_else(|_| format!("localhost:8080"));
    let external_base: String =
        env::var("EXTERNAL_BASE").unwrap_or_else(|_| format!("http://localhost:8080"));

    let database: String = env::var("DATABASE").unwrap_or_else(|_| {
        "postgresql://apevents_app:password@127.0.0.1/apevents_dev".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database)
        .await?;

    Ok(MyState {
        domain: domain.to_owned(),
        external_base: external_base.to_owned(),
        database: database.to_owned(),
        pool,
    })
}
