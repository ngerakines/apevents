use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    #[serde(rename = "id")]
    pub ap_id: String,

    pub owner: String,

    #[serde(rename = "publicKeyPem")]
    pub public_key_pem: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorMedia {
    #[serde(rename = "type")]
    pub kind: String,

    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorAttachment {
    #[serde(rename = "type")]
    pub kind: String,

    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        path.push("resources/test/thegem-city-nick.json");

        let mut file = File::open(path).expect("");
        let mut data = String::new();
        file.read_to_string(&mut data).expect("");

        let json: Actor = serde_json::from_str(&data).expect("");
        assert_eq!(
            json.endpoints.get("sharedInbox").unwrap(),
            "https://thegem.city/inbox"
        );
    }
}
