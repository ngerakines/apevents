use crate::{
    activities::{accept::Accept, follow::Follow},
    error::ApEventsError,
    state::MyStateHandle,
};
use activitypub_federation::{
    core::{
        object_id::ObjectId,
        signatures::{Keypair, PublicKey},
    },
    data::Data,
    traits::{ActivityHandler, Actor, ApubObject},
};
use activitystreams_kinds::actor::PersonType;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone)]
pub struct MyUser {
    pub ap_id: ObjectId<MyUser>,
    pub inbox: Url,
    // exists for all users (necessary to verify http signatures)
    public_key: String,
    // exists only for local users
    // private_key: Option<String>,
    pub followers: Vec<Url>,
    pub local: bool,
}

/// List of all activities which this actor can receive.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum PersonAcceptedActivities {
    Follow(Follow),
    Accept(Accept),
}

impl MyUser {
    pub fn new(ap_id: Url, keypair: Keypair) -> MyUser {
        let mut inbox = ap_id.clone();
        inbox.set_path("/inbox");
        let ap_id = ObjectId::new(ap_id);
        MyUser {
            ap_id,
            inbox,
            public_key: keypair.public_key,
            // private_key: Some(keypair.private_key),
            followers: vec![],
            local: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(rename = "type")]
    kind: PersonType,
    id: ObjectId<MyUser>,
    inbox: Url,
    public_key: PublicKey,
}

impl MyUser {
    pub fn followers(&self) -> &Vec<Url> {
        &self.followers
    }

    pub fn followers_url(&self) -> Result<Url, ApEventsError> {
        Ok(Url::parse(&format!("{}/followers", self.ap_id.inner()))?)
    }

    fn public_key(&self) -> PublicKey {
        PublicKey::new_main_key(self.ap_id.clone().into_inner(), self.public_key.clone())
    }
}

#[async_trait::async_trait(?Send)]
impl ApubObject for MyUser {
    type DataType = MyStateHandle;
    type ApubType = Person;
    type DbType = MyUser;
    type Error = crate::error::ApEventsError;

    async fn read_from_apub_id(
        object_id: Url,
        data: &Self::DataType,
    ) -> Result<Option<Self>, Self::Error> {
        let found_user: (String, String, String) =
            data.find_user_by_id(&object_id.path().to_string()).await?;
        Ok(Some(MyUser::new(
            object_id,
            Keypair {
                private_key: found_user.2,
                public_key: found_user.1,
            },
        )))
    }

    async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, Self::Error> {
        Ok(Person {
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
        Ok(MyUser {
            ap_id: apub.id,
            inbox: apub.inbox,
            public_key: apub.public_key.public_key_pem,
            // private_key: None,
            followers: vec![],
            local: false,
        })
    }
}

impl Actor for MyUser {
    fn public_key(&self) -> &str {
        &self.public_key
    }

    fn inbox(&self) -> Url {
        self.inbox.clone()
    }
}
