use std::collections::HashMap;

use crate::{
    activities::{accept::Accept, follow::Follow},
    ap::{
        self,
        actor::{Actor as ActPubActor, PublicKey as ActorPublicKey},
        ids::{generate_object_id, KindType},
    },
    error::ApEventsError,
    fed::actor_maybe,
    state::MyStateHandle,
};
use activitypub_federation::{
    core::{activity_queue::send_activity, object_id::ObjectId, signatures::PublicKey},
    data::Data,
    deser::context::WithContext,
    traits::{ActivityHandler, ApubObject, Actor},
    LocalInstance,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct EventActor {
    pub ap_id: ObjectId<EventActor>,
    pub actor_ref: String,
    pub inbox: Url,
    pub public_key_id: String,
    pub public_key: String,
    pub private_key: Option<String>,

    #[serde(skip_deserializing)]
    pub followers: Vec<Url>,

    #[serde(skip_deserializing)]
    pub local: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum PersonAcceptedActivities {
    Follow(Follow),
    Accept(Accept),
}

impl EventActor {
    pub fn followers(&self) -> &Vec<Url> {
        &self.followers
    }

    pub fn followers_url(&self) -> Result<Url, ApEventsError> {
        Ok(Url::parse(&format!("{}/followers", self.ap_id.inner()))?)
    }

    pub fn following_url(&self) -> Result<Url, ApEventsError> {
        Ok(Url::parse(&format!("{}/following", self.ap_id.inner()))?)
    }

    fn public_key(&self) -> PublicKey {
        PublicKey::new_main_key(self.ap_id.clone().into_inner(), self.public_key.clone())
    }

    pub async fn follow(
        &self,
        other: String,
        app_state: &MyStateHandle,
    ) -> Result<(), ApEventsError> {
        let found_remote_actor =
            actor_maybe(app_state, self.ap_id.to_string(), other.clone()).await?;

        let id = generate_object_id(&app_state.external_base, KindType::Follow)?;
        let follow = Follow::new(
            self.ap_id.clone(),
            found_remote_actor.ap_id.clone(),
            id.clone(),
        );

        sqlx::query(
            "INSERT INTO follow_activities (follower_ap_id, followee_ap_id, activity_ap_id) VALUES ($1, $2, $3)",
        )
            .bind(self.ap_id.to_string())
            .bind(&other)
            .bind(id.to_string())
            .execute(&app_state.pool)
            .await?;

        self.send(
            follow,
            vec![found_remote_actor.shared_inbox_or_inbox()],
            &app_state.local_instance,
        )
        .await?;
        Ok(())
    }

    pub(crate) async fn send<Activity>(
        &self,
        activity: Activity,
        recipients: Vec<Url>,
        local_instance: &LocalInstance,
    ) -> Result<(), <Activity as ActivityHandler>::Error>
    where
        Activity: ActivityHandler + Serialize,
        <Activity as ActivityHandler>::Error: From<anyhow::Error> + From<serde_json::Error>,
    {
        let activity = WithContext::new_default(activity);
        send_activity(
            activity,
            self.public_key(),
            self.private_key.clone().expect("has private key"),
            recipients,
            local_instance,
        )
        .await?;
        Ok(())
    }
}

impl TryFrom<ap::actor::Actor> for EventActor {
    type Error = ApEventsError;

    fn try_from(actor: ap::actor::Actor) -> Result<Self, Self::Error> {
        let public_key = actor.public_key.unwrap();
        Ok(EventActor {
            ap_id: ObjectId::new(Url::parse(&actor.ap_id)?),
            actor_ref: "".to_string(),
            public_key_id: public_key.ap_id,
            public_key: public_key.public_key_pem,
            private_key: None,
            inbox: Url::parse(&actor.inbox.unwrap())?,
            followers: vec![],
            local: true,
        })
    }
}

impl FromRow<'_, PgRow> for EventActor {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        let ap_id = Url::parse(row.try_get("ap_id")?).expect("msg");

        Ok(Self {
            ap_id: ObjectId::new(ap_id),
            actor_ref: row.try_get("actor_ref")?,
            public_key_id: row.try_get("public_key_id")?,
            public_key: row.try_get("public_key")?,
            private_key: row.try_get("private_key")?,
            inbox: Url::parse(row.try_get("inbox_id")?).expect("msg"),
            followers: vec![],
            local: row.try_get("is_local")?,
        })
    }
}

#[async_trait::async_trait(?Send)]
impl ApubObject for EventActor {
    type DataType = MyStateHandle;
    type ApubType = ActPubActor;
    type DbType = EventActor;
    type Error = crate::error::ApEventsError;

    async fn read_from_apub_id(
        object_id: Url,
        data: &Self::DataType,
    ) -> Result<Option<Self>, Self::Error> {
        let found_actor: EventActor = sqlx::query_as("SELECT * FROM actors WHERE ap_id = $1")
            .bind(&object_id.to_string())
            .fetch_one(&data.pool)
            .await
            .map_err(|err| ApEventsError::ActorNotFound(object_id.to_string(), err.into()))?;
        Ok(Some(found_actor))
    }

    async fn into_apub(self, data: &Self::DataType) -> Result<ActPubActor, Self::Error> {
        let ap_id = self.ap_id.to_string();
        let actor_ref_parts: Vec<&str> = self.actor_ref.split('@').collect();

        Ok(ActPubActor {
            ap_id: ap_id.clone(),
            kind: "Person".to_string(),
            following: Some(self.following_url()?.to_string()),
            followers: Some(self.followers_url()?.to_string()),
            inbox: Some(self.inbox().to_string()),
            outbox: None,
            featured: None,
            featured_tags: None,
            name: actor_ref_parts[0].to_string(),
            preferred_username: Some(actor_ref_parts[0].to_string()),
            summary: None,
            url: Some(format!("{}@{}", data.external_base, actor_ref_parts[0])),
            discoverable: Some(false),
            published: None,
            public_key: Some(ActorPublicKey {
                ap_id: self.public_key_id,
                owner: ap_id,
                public_key_pem: self.public_key,
            }),
            attachments: vec![],
            endpoints: HashMap::from([(
                "sharedInbox".to_string(),
                format!("{}/inbox", data.external_base),
            )]),
            icon: None,
            image: None,
        })
    }

    async fn verify(
        _apub: &Self::ApubType,
        _expected_domain: &Url,
        _data: &Self::DataType,
        _request_counter: &mut i32,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn from_apub(
        apub: Self::ApubType,
        data: &Self::DataType,
        _request_counter: &mut i32,
    ) -> Result<Self, Self::Error> {
        Ok(sqlx::query_as("SELECT * FROM actors WHERE ap_id = $1")
            .bind(apub.ap_id)
            .fetch_one(&data.pool)
            .await?)
    }
}

impl Actor for EventActor {
    fn public_key(&self) -> &str {
        &self.public_key
    }

    fn inbox(&self) -> Url {
        self.inbox.clone()
    }
}
