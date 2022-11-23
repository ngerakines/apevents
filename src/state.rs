use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

use crate::error::ApEventsError;

#[derive(Clone, Debug)]
pub struct MyState {
    pub domain: String,
    pub https: bool,
    pub database: String,
    pub pool: Pool<Postgres>,
}

pub async fn state_factory() -> Result<MyState, ApEventsError> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let domain: String =
        env::var("APEVENTS_DOMAIN").unwrap_or_else(|_| format!("localhost:{port}"));

    let database: String = env::var("DATABASE").unwrap_or_else(|_| {
        "postgresql://apevents_app:password@127.0.0.1/apevents_dev".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database)
        .await?;

    Ok(MyState {
        domain: domain.to_owned(),
        https: false,
        database: database.to_owned(),
        pool
    })
}
