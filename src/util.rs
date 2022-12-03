use activitypub_federation::{
    core::signatures::PublicKey, Error, LocalInstance, APUB_JSON_CONTENT_TYPE,
};
use actix_web::guard::{Guard, GuardContext};
use http_signature_normalization_reqwest::{prelude::SignExt, Config};
use log::info;
use reqwest::{header, StatusCode};

#[allow(non_snake_case)]
pub fn HeaderStart(name: &'static str, value: &'static str) -> impl Guard {
    HeaderStartGuard(
        header::HeaderName::try_from(name).unwrap(),
        value.to_string(),
        // header::HeaderValue::from_static(value),
    )
}

struct HeaderStartGuard(header::HeaderName, String);

impl Guard for HeaderStartGuard {
    fn check(&self, ctx: &GuardContext<'_>) -> bool {
        if let Some(val) = ctx.head().headers.get(&self.0) {
            let values: Vec<String> = val
                .to_str()
                .unwrap()
                .split(",")
                .map(|s| s.to_string())
                .collect();
            return values.contains(&self.1);
        }
        false
    }
}

use http::{header::HeaderName, HeaderMap, HeaderValue};
use httpdate::fmt_http_date;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::Client;
use reqwest_middleware::ClientWithMiddleware;
use serde::de::DeserializeOwned;
use std::time::{Duration, SystemTime};
use url::{ParseError, Url};

/// Just generate random url as object id. In a real project, you probably want to use
/// an url which contains the database id for easy retrieval (or store the random id in db).
pub fn generate_object_id(hostname: &str) -> Result<Url, ParseError> {
    let id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    Url::parse(&format!("http://{}/objects/{}", hostname, id))
}

pub async fn fetch_object_http<Kind: DeserializeOwned>(
    url: &Url,
    public_key_id: String,
    private_key: String,
) -> Result<Kind, Error> {
    // dont fetch local objects this way

    // verify_url_valid(url, &instance.settings).await?;
    info!("Fetching remote object {}", url.to_string());

    let client: ClientWithMiddleware = Client::default().into();
    let request_timeout = Duration::from_secs(10);

    let request_builder = client
        .get(url.to_string())
        .timeout(request_timeout)
        .headers(generate_request_headers(&url));

    let request =
        sign_request(request_builder, url.to_string(), public_key_id, private_key).await?;

    info!("request {:?}", &request);
    let res = client.execute(request).await.map_err(Error::conv)?;

    info!("response {:?}", &res);
    info!("response body {:?}", &res.text().await.unwrap());
    // let res = client
    //     .get(url.as_str())
    //     .header("Accept", APUB_JSON_CONTENT_TYPE)
    //     .timeout(request_timeout)
    //     .send()
    //     .await
    //     .map_err(Error::conv)?;

    // if res.status() == StatusCode::GONE {
    //     return Err(Error::ObjectDeleted);
    // }

    // res.json().await.map_err(Error::conv)
    Err(Error::ObjectDeleted)
}

fn generate_request_headers(inbox_url: &Url) -> HeaderMap {
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

use openssl::{
    hash::MessageDigest,
    pkey::PKey,
    rsa::Rsa,
    sign::{Signer, Verifier},
};
use reqwest::Request;
use reqwest_middleware::RequestBuilder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
