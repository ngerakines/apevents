use std::collections::HashMap;

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{error::ApEventsError, state::MyStateHandle, storage_domains::list_allowed_domains};

#[derive(Serialize, Deserialize)]
pub struct NodeInfoLinks {
    pub links: Vec<NodeInfoLink>,
}

#[derive(Serialize, Deserialize)]
pub struct NodeInfoLink {
    pub rel: String,
    pub href: String,
}

#[derive(Serialize, Deserialize)]
pub struct NodeInfo20Software {
    name: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
pub struct NodeInfo20Usage {
    users: HashMap<String, u32>,
    local_posts: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo20 {
    version: String,
    software: NodeInfo20Software,
    protocols: Vec<String>,
    services: HashMap<String, Vec<String>>,
    usage: NodeInfo20Usage,
    open_registrations: bool,
    metadata: HashMap<String, String>,
}

pub async fn handle_wellknown_nodeinfo(
    app_state: web::Data<MyStateHandle>,
) -> Result<HttpResponse, ApEventsError> {
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(NodeInfoLinks {
            links: vec![NodeInfoLink {
                rel: "http://nodeinfo.diaspora.software/ns/schema/2.0".to_string(),
                href: format!("{}/nodeinfo/2.0", app_state.external_base),
            }],
        }))
}

pub async fn handle_nodeinfo_20() -> Result<HttpResponse, ApEventsError> {
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(NodeInfo20 {
            version: "2.0".to_string(),
            software: NodeInfo20Software {
                name: "apevents".to_string(),
                version: "0.1.0-alpa".to_string(),
            },
            protocols: vec!["activitypub".to_string()],
            services: HashMap::from([
                ("outbound".to_string(), vec![]),
                ("inbound".to_string(), vec![]),
            ]),
            usage: NodeInfo20Usage {
                local_posts: 0,
                users: HashMap::from([
                    ("total".to_string(), 1),
                    ("activeMonth".to_string(), 1),
                    ("activeHalfyear".to_string(), 1),
                ]),
            },
            open_registrations: false,
            metadata: HashMap::from([]),
        }))
}

pub async fn handle_instance_peers(app_state: web::Data<MyStateHandle>) -> Result<HttpResponse, ApEventsError> {
    let domains = list_allowed_domains(&app_state).await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(domains))
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InstanceInfo {
    uri: String,
    title: String,
    short_description: String,
    description: String,
    email: String,
    version: String,
    urls: HashMap<String, String>,
    stats: HashMap<String, u32>,
    thumbnail: String,
    languages: Vec<String>,
    registrations: bool,
    approval_required: bool,
    invites_enabled: bool,
    configuration: HashMap<String, String>,
    contact_account: HashMap<String, String>,
    rules: Vec<String>,
}

pub async fn handle_instance_info_v1(
    app_state: web::Data<MyStateHandle>,
) -> Result<HttpResponse, ApEventsError> {
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(InstanceInfo {
            uri: app_state.domain.clone(),
            title: "APEvents".to_string(),
            short_description: "An event framework".to_string(),
            description: "APEvents is an open source federated event framework.".to_string(),
            email: "admin@thegem.city".to_string(),
            version: "0.1.0-alpha (compatible; apevents)".to_string(),
            urls: HashMap::new(),
            stats: HashMap::from([
                ("user_count".to_string(), 1),
                ("status_count".to_string(), 1),
                ("domain_count".to_string(), 1),
            ]),
            thumbnail: format!("{}/thumbnail.png", app_state.external_base),
            languages: vec!["en".to_string()],
            registrations: false,
            approval_required: true,
            invites_enabled: true,
            configuration: HashMap::new(),
            contact_account: HashMap::new(),
            rules: vec![],
        }))
}
