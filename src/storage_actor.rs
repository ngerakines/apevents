use reqwest::Url;
use sqlx::{Postgres, QueryBuilder};

use crate::{
    ap::actor::Actor, error::ApEventsError, objects::actor::EventActor, state::MyStateHandle,
};

pub async fn create_actor(
    app_state: &MyStateHandle,
    actor: Actor,
    private_key: Option<String>,
) -> Result<EventActor, ApEventsError> {
    let parsed_resource = Url::parse(actor.ap_id.clone().as_str())?;
    let domain = parsed_resource
        .domain()
        .ok_or_else(|| ApEventsError::new("invalid domain".to_string()))?;

    let mut tx = app_state.pool.begin().await?;

    let ap_id = &actor.ap_id;

    insert_actor_query(&actor, domain)?
        .build()
        .execute(&mut tx)
        .await
        .map_err(ApEventsError::conv)?;

    if actor.url.is_some() {
        sqlx::query(
            "UPDATE actors SET resources = array_append(resources, $2::varchar) WHERE ap_id = $1",
        )
        .bind(ap_id)
        .bind(actor.url)
        .execute(&mut tx)
        .await
        .map_err(ApEventsError::conv)?;
    }

    if private_key.is_some() {
        sqlx::query("UPDATE actors SET private_key = $2 WHERE ap_id = $1")
            .bind(ap_id)
            .bind(private_key.unwrap())
            .execute(&mut tx)
            .await
            .map_err(ApEventsError::conv)?;
    }

    tx.commit().await.map_err(ApEventsError::conv)?;

    let found_actor: EventActor = sqlx::query_as("SELECT * FROM actors WHERE ap_id = $1")
        .bind(ap_id)
        .fetch_one(&app_state.pool)
        .await
        .map_err(ApEventsError::conv)?;

    Ok(found_actor)
}

pub fn insert_actor_query<'a>(
    actor: &'a Actor,
    domain: &str,
) -> Result<QueryBuilder<'a, Postgres>, ApEventsError> {
    if actor.inbox.is_none() {
        return Err(ApEventsError::new("actor inbox missing".to_string()));
    }

    if actor.public_key.is_none() {
        return Err(ApEventsError::new("actor public_key missing".to_string()));
    }
    let public_key = (actor.public_key.as_ref()).unwrap();

    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new("INSERT INTO actors (");

    let mut fields = query_builder.separated(", ");
    fields.push("ap_id");
    fields.push("actor_ref");
    fields.push("is_local");
    fields.push("inbox_id");
    fields.push("public_key_id");
    fields.push("public_key");
    fields.push("resources");
    fields.push_unseparated(") ");

    query_builder.push("VALUES (");

    let mut values = query_builder.separated(", ");
    values.push_bind(actor.ap_id.clone());
    values.push_bind(format!("{}@{}", actor.name, domain));
    values.push_bind(false);
    values.push_bind(actor.inbox.as_ref().unwrap());
    values.push_bind(public_key.ap_id.clone());
    values.push_bind(public_key.public_key_pem.clone());
    values.push("ARRAY[$1, $2]");

    values.push_unseparated(")");

    Ok(query_builder)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ap::actor::{Actor, ActorAttachment, PublicKey};

    use super::*;

    #[test]
    fn query_builder() {
        let cmp_actor = Actor {
            ap_id: "https://thegem.city/users/nick".to_string(),
            kind: "Person".to_string(),
            following: Some("https://thegem.city/users/nick/following".to_string()),
            followers: Some("https://thegem.city/users/nick/followers".to_string()),
            inbox: Some("https://thegem.city/users/nick/inbox".to_string()),
            outbox: Some("https://thegem.city/users/nick/outbox".to_string()),
            featured: Some("https://thegem.city/users/nick/collections/featured".to_string()),
            featured_tags: Some("https://thegem.city/users/nick/collections/tags".to_string()),
            name: "Nick Gerakines".to_string(),
            preferred_username: Some("nick".to_string()),
            summary: Some("<p>thegem.city admin | Previously DataDog, Mattel, Blizzard, EA, Yahoo, 6A | Worker | Swing dancer | Taco enthusiast | Elder Millennial | Probably the best Nick there is | Engaged to <span class=\"h-card\"><a href=\"https://thegem.city/@mattie\" class=\"u-url mention\">@<span>mattie</span></a></span> | he/him :bisexual_flag:</p><p><a href=\"https://thegem.city/tags/fedi22\" class=\"mention hashtag\" rel=\"tag\">#<span>fedi22</span></a> <a href=\"https://thegem.city/tags/ohio\" class=\"mention hashtag\" rel=\"tag\">#<span>ohio</span></a> <a href=\"https://thegem.city/tags/devops\" class=\"mention hashtag\" rel=\"tag\">#<span>devops</span></a> <a href=\"https://thegem.city/tags/mastodon\" class=\"mention hashtag\" rel=\"tag\">#<span>mastodon</span></a> <a href=\"https://thegem.city/tags/aiml\" class=\"mention hashtag\" rel=\"tag\">#<span>aiml</span></a> <a href=\"https://thegem.city/tags/python\" class=\"mention hashtag\" rel=\"tag\">#<span>python</span></a> <a href=\"https://thegem.city/tags/programming\" class=\"mention hashtag\" rel=\"tag\">#<span>programming</span></a> <a href=\"https://thegem.city/tags/lgbt\" class=\"mention hashtag\" rel=\"tag\">#<span>lgbt</span></a></p>".to_string()),
            url: Some("https://thegem.city/@nick".to_string()),
            discoverable: Some(true),
            published: Some("2022-11-02T00:00:00Z".to_string()),
            public_key: Some(PublicKey { ap_id: "https://thegem.city/users/nick#main-key".to_string(), owner: "https://thegem.city/users/nick".to_string(),
            public_key_pem: "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAoYtjU511NUxW3YnFYEsZ\nAKIrh8La21ZB4UKPyXMckhUpHWM1wEbWJ8Ql014shOBUSLO4w4i4/zl+LJrOO5eT\nVhXYl9+8wxZjLqzv0DZLQCD8qn244nrcSndqtW2oC4F/5781UwoTU+TFs2ODcfjS\nQjI2yagS4uRlJxu7wQN/w0Zi/kLbC459Xts6Vnz5Lj7qTpEvBuyZW71j2sVR0e/N\nf9P+E5P9K4RPzeY6q03GIkiZh9lW7F+6Dv+86bU7a8eyBQlkr64Nyoz3rcxo5Vk2\n0GOpv87i+9Di2tPG/qUbmgM89N/PVsctIDTFCsOlrxppay/8qdR1HuU53nqw/EVo\n7QIDAQAB\n-----END PUBLIC KEY-----\n".to_string() }),
            attachments: vec![
                ActorAttachment { kind: "PropertyValue".to_string(), name: "pronouns".to_string(), value: "he/him".to_string() },
                ActorAttachment { kind: "PropertyValue".to_string(), name: "keyoxide".to_string(), value: "openpgp4fpr:8298D48681C260B66A4FFA59FD9B9F77EA58CD5F".to_string() }
            ],
            endpoints: HashMap::from([("sharedInbox".to_string(), "https://thegem.city/inbox".to_string())]),
            icon: None,
            image: None
        };

        assert_eq!(insert_actor_query(&cmp_actor, "thegem.city").expect("query built").sql(), "INSERT INTO actors (ap_id, actor_ref, is_local, inbox_id, public_key_id, public_key, resources) VALUES ($1, $2, $3, $4, $5, $6, ARRAY[$1, $2])");
    }
}
