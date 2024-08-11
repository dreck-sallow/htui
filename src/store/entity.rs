use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum RequestMethod {
    Head,
    Get,
    Post,
    Put,
    Connect,
    Trace,
    Delete,
    Patch,
    Options,
}

#[derive(Serialize, Deserialize)]
pub struct Request {
    name: String,
    method: RequestMethod,
    url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Collection {
    name: String,
    requests: Vec<Request>,
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    name: String,
    collections: Vec<Collection>,
}
