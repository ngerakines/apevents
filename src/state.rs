use activitypub_federation::core::signatures::generate_actor_keypair;
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    Pool, Postgres, Row,
};
use std::{env, sync::Arc};
use tracing::info;

use crate::error::ApEventsError;

pub type MyStateHandle = Arc<MyState>;

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

#[derive(Debug)]
pub struct User {
    pub name: String,
    pub public_key: String,
    pub private_key: String,
}

impl<'r> sqlx::FromRow<'r, PgRow> for User {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(User {
            name: row.get("name"),
            public_key: row.try_get("name")?,
            private_key: row.try_get("name")?,
        })
    }
}

impl MyState {
    pub async fn create_user(&self) -> Result<String, ApEventsError> {
        let name = petname::Petnames::default().generate_one(3, "-");
        let object_id = format!("/users/{}", name);

        let keypair = generate_actor_keypair()?;

        sqlx::query("INSERT INTO USERS (name, domain, public_key, private_key, object_id) values ($1, $2, $3, $4, $5)")
            .bind(name.clone())
            .bind(&self.domain)
            .bind(keypair.public_key)
            .bind(keypair.private_key)
            .bind(object_id)
            .execute(&self.pool)
            .await?;

        Ok(name.clone())
    }

    pub async fn find_user_by_id(
        &self,
        name: &String,
    ) -> Result<(String, String, String), ApEventsError> {
        info!("querying {}", name);
        let results: (String, String, String) = sqlx::query_as(
            "SELECT name, public_key, private_key FROM users WHERE object_id = $1 AND domain = $2",
        )
        .bind(name)
        .bind(&self.domain)
        .fetch_one(&self.pool)
        .await?;

        Ok(results)
    }
}
