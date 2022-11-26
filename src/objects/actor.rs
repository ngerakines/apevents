use crate::{
    activities::{accept::Accept, follow::Follow},
    error::ApEventsError,
    state::MyStateHandle,
};
use activitypub_federation::{
    core::{activity_queue::send_activity, object_id::ObjectId, signatures::PublicKey},
    data::Data,
    deser::context::WithContext,
    traits::{ActivityHandler, Actor, ApubObject},
    LocalInstance,
};
use activitystreams_kinds::actor::PersonType;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};
use url::Url;

#[derive(Debug, Clone)]
pub struct EventActor {
    pub ap_id: ObjectId<EventActor>,
    pub inbox: Url,
    public_key: String,
    private_key: Option<String>,
    pub followers: Vec<Url>,
    pub local: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventActorView {
    #[serde(rename = "type")]
    kind: PersonType,
    id: ObjectId<EventActor>,
    inbox: Url,
    public_key: PublicKey,
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

    fn public_key(&self) -> PublicKey {
        PublicKey::new_main_key(self.ap_id.clone().into_inner(), self.public_key.clone())
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

impl FromRow<'_, PgRow> for EventActor {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        let ap_id = Url::parse(row.try_get("ap_id")?).expect("msg");

        Ok(Self {
            ap_id: ObjectId::new(ap_id),
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
    type ApubType = EventActorView;
    type DbType = EventActor;
    type Error = crate::error::ApEventsError;

    async fn read_from_apub_id(
        object_id: Url,
        data: &Self::DataType,
    ) -> Result<Option<Self>, Self::Error> {
        let found_actor: EventActor = sqlx::query_as("SELECT * FROM actors WHERE ap_id = $1")
            .bind(&object_id.to_string())
            .fetch_one(&data.pool)
            .await?;
        Ok(Some(found_actor))
    }

    async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, Self::Error> {
        Ok(EventActorView {
            kind: Default::default(),
            id: self.ap_id.clone(),
            inbox: self.inbox.clone(),
            public_key: self.public_key(),
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
        _data: &Self::DataType,
        _request_counter: &mut i32,
    ) -> Result<Self, Self::Error> {
        Ok(EventActor {
            ap_id: apub.id,
            inbox: apub.inbox,
            public_key: apub.public_key.public_key_pem,
            private_key: None,
            followers: vec![],
            local: false,
        })
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
