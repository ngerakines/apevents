use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublicKey {
    #[serde(rename = "id")]
    pub ap_id: String,

    pub owner: String,

    #[serde(rename = "publicKeyPem")]
    pub public_key_pem: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActorMedia {
    #[serde(rename = "type")]
    pub kind: String,

    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActorAttachment {
    #[serde(rename = "type")]
    pub kind: String,

    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Actor {
    #[serde(rename = "id")]
    pub ap_id: String,

    #[serde(rename = "type")]
    pub kind: String,

    pub following: Option<String>,
    pub followers: Option<String>,
    pub inbox: Option<String>,
    pub outbox: Option<String>,
    pub featured: Option<String>,

    #[serde(rename = "featuredTags")]
    pub featured_tags: Option<String>,

    pub name: String,

    #[serde(rename = "preferredUsername")]
    pub preferred_username: Option<String>,

    pub summary: Option<String>,
    pub url: Option<String>,

    pub discoverable: Option<bool>,
    pub published: Option<String>,

    #[serde(rename = "publicKey")]
    pub public_key: Option<PublicKey>,

    #[serde(rename = "attachment")]
    pub attachments: Vec<ActorAttachment>,

    pub endpoints: HashMap<String, String>,

    pub icon: Option<ActorMedia>,
    pub image: Option<ActorMedia>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{fs::File, io::Read, path::PathBuf};

    #[test]
    fn load_profile() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/mastodon-4.0.0-thegem-city-nick.json");

        let mut file = File::open(path).expect("");
        let mut data = String::new();
        file.read_to_string(&mut data).expect("");

        let json: Actor = serde_json::from_str(&data).expect("");
        assert_eq!(json, Actor {
            ap_id: "https://thegem.city/users/nick".to_string(),
             kind: "Person".to_string(),
             following: Some("https://thegem.city/users/nick/following".to_string()),
             followers: Some("https://thegem.city/users/nick/followers".to_string()),
              inbox: Some("https://thegem.city/users/nick/inbox".to_string()),
               outbox: Some("https://thegem.city/users/nick/outbox".to_string()),
               featured: Some("https://thegem.city/users/nick/collections/featured".to_string()),
               featured_tags: Some("https://thegem.city/users/nick/collections/tags".to_string()),
               name: "Nick Gerakines".to_string(),
               preferred_username: Some("nick".to_string()),
               summary: Some("<p>thegem.city admin | Previously DataDog, Mattel, Blizzard, EA, Yahoo, 6A | Worker | Swing dancer | Taco enthusiast | Elder Millennial | Probably the best Nick there is | Engaged to <span class=\"h-card\"><a href=\"https://thegem.city/@mattie\" class=\"u-url mention\">@<span>mattie</span></a></span> | he/him :bisexual_flag:</p><p><a href=\"https://thegem.city/tags/fedi22\" class=\"mention hashtag\" rel=\"tag\">#<span>fedi22</span></a> <a href=\"https://thegem.city/tags/ohio\" class=\"mention hashtag\" rel=\"tag\">#<span>ohio</span></a> <a href=\"https://thegem.city/tags/devops\" class=\"mention hashtag\" rel=\"tag\">#<span>devops</span></a> <a href=\"https://thegem.city/tags/mastodon\" class=\"mention hashtag\" rel=\"tag\">#<span>mastodon</span></a> <a href=\"https://thegem.city/tags/aiml\" class=\"mention hashtag\" rel=\"tag\">#<span>aiml</span></a> <a href=\"https://thegem.city/tags/python\" class=\"mention hashtag\" rel=\"tag\">#<span>python</span></a> <a href=\"https://thegem.city/tags/programming\" class=\"mention hashtag\" rel=\"tag\">#<span>programming</span></a> <a href=\"https://thegem.city/tags/lgbt\" class=\"mention hashtag\" rel=\"tag\">#<span>lgbt</span></a></p>".to_string()),
               url: Some("https://thegem.city/@nick".to_string()),
               discoverable: Some(true),
               published: Some("2022-11-02T00:00:00Z".to_string()),
               public_key: Some(PublicKey { ap_id: "https://thegem.city/users/nick#main-key".to_string(), owner: "https://thegem.city/users/nick".to_string(),
               public_key_pem: "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAoYtjU511NUxW3YnFYEsZ\nAKIrh8La21ZB4UKPyXMckhUpHWM1wEbWJ8Ql014shOBUSLO4w4i4/zl+LJrOO5eT\nVhXYl9+8wxZjLqzv0DZLQCD8qn244nrcSndqtW2oC4F/5781UwoTU+TFs2ODcfjS\nQjI2yagS4uRlJxu7wQN/w0Zi/kLbC459Xts6Vnz5Lj7qTpEvBuyZW71j2sVR0e/N\nf9P+E5P9K4RPzeY6q03GIkiZh9lW7F+6Dv+86bU7a8eyBQlkr64Nyoz3rcxo5Vk2\n0GOpv87i+9Di2tPG/qUbmgM89N/PVsctIDTFCsOlrxppay/8qdR1HuU53nqw/EVo\n7QIDAQAB\n-----END PUBLIC KEY-----\n".to_string() }),
               attachments: vec![
                ActorAttachment { kind: "PropertyValue".to_string(), name: "pronouns".to_string(), value: "he/him".to_string() },
                ActorAttachment { kind: "PropertyValue".to_string(), name: "keyoxide".to_string(), value: "openpgp4fpr:8298D48681C260B66A4FFA59FD9B9F77EA58CD5F".to_string() }
            ],
               endpoints: HashMap::from([("sharedInbox".to_string(), "https://thegem.city/inbox".to_string())]),
               icon: Some(ActorMedia {
                kind: "Image".to_string(), media_type: "image/jpeg".to_string(), url: "https://s3-us-east-2.amazonaws.com/thegem-city-assets/accounts/avatars/109/272/112/841/303/240/original/6ad687938d3a1af9.jpg".to_string() }),
               image: Some(ActorMedia { kind: "Image".to_string(), media_type: "image/png".to_string(),
                url: "https://s3-us-east-2.amazonaws.com/thegem-city-assets/accounts/headers/109/272/112/841/303/240/original/0a834dbf6f05f8b8.png".to_string()
            })
        });
    }
}
