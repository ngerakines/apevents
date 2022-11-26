use activitypub_federation::UrlVerifier;
use async_trait::async_trait;
use url::Url;

/// Use this to store your federation blocklist, or a database connection needed to retrieve it.
#[derive(Clone)]
pub struct MyUrlVerifier();

#[async_trait]
impl UrlVerifier for MyUrlVerifier {
    async fn verify(&self, _: &Url) -> Result<(), &'static str> {
        Ok(())
    }
}
