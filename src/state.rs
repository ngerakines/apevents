use activitypub_federation::{InstanceSettings, LocalInstance};
use reqwest::Client;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{env, sync::Arc};

use crate::error::ApEventsError;
use crate::instance::MyUrlVerifier;

pub type MyStateHandle = Arc<MyState>;

pub struct MyState {
    pub domain: String,
    pub external_base: String,
    pub database: String,
    pub local_instance: LocalInstance,

    pub pool: Pool<Postgres>,
}

pub async fn state_factory() -> Result<MyStateHandle, ApEventsError> {
    let domain: String = env::var("DOMAIN").unwrap_or_else(|_| "localhost:8080".to_string());
    let external_base: String =
        env::var("EXTERNAL_BASE").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let database: String = env::var("DATABASE").unwrap_or_else(|_| {
        "postgresql://apevents_app:password@127.0.0.1/apevents_dev".to_string()
    });

    let settings = InstanceSettings::builder()
        .debug(true)
        .url_verifier(Box::new(MyUrlVerifier()))
        .build()?;

    let local_instance = LocalInstance::new(domain.clone(), Client::default().into(), settings);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database)
        .await?;

    Ok(Arc::new(MyState {
        domain: domain.to_owned(),
        external_base: external_base.to_owned(),
        database: database.to_owned(),
        local_instance,
        pool,
    }))
}
