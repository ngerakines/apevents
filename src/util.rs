use activitypub_federation::{Error, APUB_JSON_CONTENT_TYPE};
use actix_web::guard::{Guard, GuardContext};
use http::{header::HeaderName, HeaderMap, HeaderValue};
use http_signature_normalization_reqwest::{prelude::SignExt, Config};
use httpdate::fmt_http_date;
use log::info;
use openssl::{hash::MessageDigest, pkey::PKey, sign::Signer};
use reqwest::header;
use reqwest::Client;
use reqwest::Request;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_middleware::RequestBuilder;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime};
use url::Url;

#[allow(non_snake_case)]
pub fn HeaderStart(name: &'static str, value: &'static str) -> impl Guard {
    HeaderStartGuard(
        header::HeaderName::try_from(name).unwrap(),
        value.to_string(),
    )
}

struct HeaderStartGuard(header::HeaderName, String);

impl Guard for HeaderStartGuard {
    fn check(&self, ctx: &GuardContext<'_>) -> bool {
        if let Some(val) = ctx.head().headers.get(&self.0) {
            return val
                .to_str()
                .unwrap()
                .split(',')
                .map(|s| s.to_string())
                .any(|x| x == self.1);
        }
        false
    }
}

pub async fn fetch_object_http<Kind: DeserializeOwned>(
    url: &Url,
    public_key_id: String,
    private_key: String,
) -> Result<Kind, Error> {
    // TODO: Bail if url starts with "<external_base>/"

    info!("Fetching remote object {}", url.to_string());

    let client: ClientWithMiddleware = Client::default().into();
    let request_timeout = Duration::from_secs(10);

    let request_builder = client
        .get(url.to_string())
        .timeout(request_timeout)
        .headers(generate_object_request_headers(url));

    let request =
        sign_request(request_builder, url.to_string(), public_key_id, private_key).await?;
    let res = client.execute(request).await.map_err(Error::conv)?;

    res.json().await.map_err(Error::conv)
}

fn generate_object_request_headers(inbox_url: &Url) -> HeaderMap {
    let mut host = inbox_url.domain().expect("read inbox domain").to_string();
    if let Some(port) = inbox_url.port() {
        host = format!("{}:{}", host, port);
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("accept"),
        HeaderValue::from_static(APUB_JSON_CONTENT_TYPE),
    );
    headers.insert(
        HeaderName::from_static("host"),
        HeaderValue::from_str(&host).expect("Hostname is valid"),
    );
    headers.insert(
        "date",
        HeaderValue::from_str(&fmt_http_date(SystemTime::now())).expect("Date is valid"),
    );
    headers
}

pub(crate) async fn sign_request(
    request_builder: RequestBuilder,
    activity: String,
    public_key_id: String,
    private_key: String,
) -> Result<Request, anyhow::Error> {
    let sig_conf = Config::new().mastodon_compat();
    request_builder
        .signature_with_digest(
            sig_conf.clone(),
            public_key_id,
            Sha256::new(),
            activity,
            move |signing_string| {
                let private_key = PKey::private_key_from_pem(private_key.as_bytes())?;
                let mut signer = Signer::new(MessageDigest::sha256(), &private_key)?;
                signer.update(signing_string.as_bytes())?;

                Ok(base64::encode(signer.sign_to_vec()?)) as Result<_, anyhow::Error>
            },
        )
        .await
}
