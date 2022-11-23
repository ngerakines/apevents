use actix_web::web::Data;
use actix_webfinger::{Link, Resolver, Webfinger};
use std::{future::Future, pin::Pin};

use crate::state::MyState;

pub struct MyResolver;

type LocalBoxFuture<'a, Output> = Pin<Box<dyn Future<Output = Output> + 'a>>;

impl Resolver for MyResolver {
    type State = Data<MyState>;
    type Error = actix_web::error::JsonPayloadError;

    fn find(
        scheme: Option<&str>,
        account: &str,
        domain: &str,
        state: Data<MyState>,
    ) -> LocalBoxFuture<'static, Result<Option<Webfinger>, Self::Error>> {
        let w = if scheme == Some("acct:") && domain == state.domain {
            let mut svc = Webfinger::new(&format!("acct:{}@{}", account, domain));
            svc.add_alias(&format!("https://{}/@{}", domain, account));
            svc.add_link(Link {
                rel: "http://webfinger.net/rel/profile-page".to_string(),
                kind: Some("text/html".to_string()),
                href: Some(format!("http://{}/@{}", domain, account)),
                template: None,
            });
            svc.add_link(Link {
                rel: "self".to_string(),
                kind: Some("application/activity+json".to_string()),
                href: Some(format!("http://{}/users/{}", domain, account)),
                template: None,
            });
            svc.add_link(Link {
                rel: "http://ostatus.org/schema/1.0/subscribe".to_string(),
                kind: None,
                href: None,
                template: Some(format!(
                    "http://{}/authorize_interaction?uri={{uri}}",
                    domain
                )),
            });
            Some(svc)
        } else {
            None
        };

        Box::pin(async move { Ok(w) })
    }
}
