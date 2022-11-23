use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_derive::Deserialize;
use thiserror::Error;

use actix_webfinger::{Link, Webfinger};

use crate::state::MyState;

#[derive(Clone, Debug, Error)]
#[error("Resource {0} is invalid")]
pub struct InvalidResource(String);

#[derive(Clone, Debug)]
pub struct WebfingerResource {
    pub scheme: Option<String>,
    pub account: String,
    pub domain: String,
}

impl std::str::FromStr for WebfingerResource {
    type Err = InvalidResource;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (scheme, trimmed) = s
            .find(':')
            .map(|index| {
                let (scheme, trimmed) = s.split_at(index);
                (
                    Some(scheme.to_owned() + ":"),
                    trimmed.trim_start_matches(':'),
                )
            })
            .unwrap_or((None, s));

        let trimmed = trimmed.trim_start_matches('@');

        if let Some(index) = trimmed.find('@') {
            let (account, domain) = trimmed.split_at(index);

            Ok(WebfingerResource {
                scheme,
                account: account.to_owned(),
                domain: domain.trim_start_matches('@').to_owned(),
            })
        } else {
            Err(InvalidResource(s.to_owned()))
        }
    }
}

impl<'de> serde::de::Deserialize<'de> for WebfingerResource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<WebfingerResource>()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct WebfingerQuery {
    resource: WebfingerResource,
}

pub async fn handle_webfinger(
    app_state: web::Data<MyState>,
    _req: HttpRequest,
    query: web::Query<WebfingerQuery>,
) -> impl Responder {
    let WebfingerResource {
        scheme: _,
        account,
        domain,
    } = query.into_inner().resource;

    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) as count FROM users WHERE name = $1")
        .bind(&account)
        .fetch_one(&app_state.pool)
        .await
        .unwrap_or((0,));

    if user_count == (0,) {
        return HttpResponse::NotFound().finish();
    }

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
    svc.respond()
}
