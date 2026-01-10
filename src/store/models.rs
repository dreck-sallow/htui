use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub type TimeId = String;

pub fn time_as_id() -> TimeId {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}

#[derive(Deserialize, Serialize)]
pub struct ProjectModel {
    pub id: TimeId,
    pub name: String,
    #[serde(default)]
    pub collections: Vec<CollectionModel>,
    #[serde(default)]
    pub environments: Vec<Environment>,
    #[serde(default)]
    pub selected_env_context: Option<usize>,
}

impl ProjectModel {
    pub fn new(name: String) -> Self {
        Self {
            id: time_as_id(),
            name,
            collections: Vec::new(),
            environments: Vec::new(),
            selected_env_context: None,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CollectionModel {
    pub id: TimeId,
    pub name: String,
    pub requests: Vec<RequestModel>,
}

#[derive(Deserialize, Serialize)]
pub struct RequestModel {
    pub id: TimeId,
    pub name: String,
    pub url: String,
    pub headers: Vec<KeyValueParam>,
    pub params: Vec<KeyValueParam>,
    pub method: HttpMethod,
    pub body: RequestBody,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Options,
    Get,
    Post,
    Put,
    Delete,
    Head,
    Patch,
}

#[derive(Serialize, Deserialize)]
pub enum RequestBody {
    None,
    Text(String),
    Json(serde_json::Value),
    FormUrlEncoded(Vec<(String, String)>),
    FormData(Vec<FormParam>),
}

#[derive(Serialize, Deserialize)]
pub struct FormParam {
    pub value: String,
    pub key: String,
    pub is_file: bool,
}

#[derive(Serialize, Deserialize)]
pub struct KeyValueParam {
    #[serde(default)]
    pub enable: bool,
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Environment {
    pub name: String,
    pub variables: Vec<Variable>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub value: String,
}
