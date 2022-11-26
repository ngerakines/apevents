use crate::{objects::actor::EventActor, state::MyStateHandle};
use activitypub_federation::{
    core::object_id::ObjectId,
    data::Data,
    traits::{ActivityHandler, Actor},
};
use activitystreams_kinds::activity::FollowType;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use url::Url;

use super::accept::Accept;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    pub(crate) actor: ObjectId<EventActor>,
    pub(crate) object: ObjectId<EventActor>,
    #[serde(rename = "type")]
    kind: FollowType,
    id: Url,
}

impl Follow {
    pub fn new(actor: ObjectId<EventActor>, object: ObjectId<EventActor>, id: Url) -> Follow {
        Follow {
            actor,
            object,
            kind: Default::default(),
            id,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for Follow {
    type DataType = MyStateHandle;
    type Error = crate::error::ApEventsError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(
        &self,
        _data: &Data<Self::DataType>,
        _request_counter: &mut i32,
    ) -> Result<(), Self::Error> {
        // TODO: Reject denied domains.
        // TODO: Reject denied actors.
        Ok(())
    }

    // Ignore clippy false positive: https://github.com/rust-lang/rust-clippy/issues/6446
    #[allow(clippy::await_holding_lock)]
    async fn receive(
        self,
        app_state: &Data<Self::DataType>,
        request_counter: &mut i32,
    ) -> Result<(), Self::Error> {
        // TODO: Move all of this into a background job.

        let found_actor: EventActor = sqlx::query_as("SELECT * FROM actors WHERE ap_id = $1")
            .bind(self.id.to_string())
            .fetch_one(&app_state.pool)
            .await?;

        let follower = self
            .actor
            .dereference(app_state, &app_state.local_instance, request_counter)
            .await?;

        let accept_partial: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        let accept_ap_id = Url::parse(&format!(
            "{}/object/{}",
            app_state.external_base, accept_partial
        ))?;

        sqlx::query("insert into actor_followers (actor_ap_id, follower_ap_id, accept_ap_id) values ($1, $2, $3)")
        .bind(found_actor.ap_id.to_string())
        .bind(follower.ap_id.to_string())
        .bind(accept_ap_id.path().to_string())
        .execute(&app_state.pool)
        .await?;

        let accept = Accept::new(found_actor.ap_id.clone(), self, accept_ap_id.clone());
        found_actor
            .send(
                accept,
                vec![follower.shared_inbox_or_inbox()],
                &app_state.local_instance,
            )
            .await?;
        Ok(())
    }
}
