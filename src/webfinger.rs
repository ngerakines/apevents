use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_derive::Deserialize;
use thiserror::Error;

use actix_webfinger::{Link, Webfinger};

use crate::state::MyStateHandle;

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
    app_state: web::Data<MyStateHandle>,
    _req: HttpRequest,
    query: web::Query<WebfingerQuery>,
) -> impl Responder {
    let WebfingerResource {
        scheme,
        account,
        domain,
    } = query.into_inner().resource;

    if Some("acct:".to_string()) != scheme {
        return HttpResponse::NotFound().finish();
    }

    if domain != app_state.domain {
        return HttpResponse::NotFound().finish();
    }

    let ap_id = format!("{}/actor/{}",app_state.external_base, account);

    let user_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) as count FROM actors WHERE ap_id = $1")
            .bind(&ap_id)
            .fetch_one(&app_state.pool)
            .await
            .unwrap_or((0,));

    if user_count == (0,) {
        return HttpResponse::NotFound().finish();
    }

    let mut svc = Webfinger::new(&format!("acct:{}@{}", account, app_state.domain));
    svc.add_alias(&format!("{}/@{}", app_state.external_base, account));
    svc.add_link(Link {
        rel: "http://webfinger.net/rel/profile-page".to_string(),
        kind: Some("text/html".to_string()),
        href: Some(format!("{}/@{}", app_state.external_base, account)),
        template: None,
    });
    svc.add_link(Link {
        rel: "self".to_string(),
        kind: Some("application/activity+json".to_string()),
        href: Some(format!("{}/actor/{}", app_state.external_base, account)),
        template: None,
    });
    svc.add_link(Link {
        rel: "http://ostatus.org/schema/1.0/subscribe".to_string(),
        kind: None,
        href: None,
        template: Some(format!(
            "{}/authorize_interaction?uri={{uri}}",
            app_state.external_base
        )),
    });
    svc.respond()
}
