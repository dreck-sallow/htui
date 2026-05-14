use std::{collections::HashMap, time::Duration};

use crate::store::models::TimeId;

use super::table::{TableRow, TableState};

pub struct Responses {
    pub list: HashMap<TimeId, ResponseStatus>,
}

impl Responses {
    pub fn new() -> Self {
        Self {
            list: HashMap::new(),
        }
    }
}

pub enum ResponseStatus {
    Fetching,
    Success(Response),
    Error(String),
    Cancelled,
}

pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub version: String,
    pub duration: Duration,
    pub size_bytes: usize,
    pub content_type: String,
    pub headers: TableState<ReadonlyHeader>,
    pub cookies: TableState<Cookie>,
    pub body: ResponseBody,
}

pub enum ResponseBody {
    Text(String),
    Binary(Vec<u8>),
    Empty,
}

pub struct ReadonlyHeader {
    pub key: String,
    pub value: String,
}

impl TableRow for ReadonlyHeader {
    fn next_cell(&self, idx: Option<usize>) -> Option<usize> {
        Self::next_with_limit(&self, idx, 1)
    }
}

pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub expires: Option<String>,
    pub max_ge: Option<String>,
    pub path: Option<String>,
    pub http_only: bool,
    pub partitioned: bool,
    pub secure: bool,
    pub same_site: Option<String>,
}

impl TableRow for Cookie {
    fn next_cell(&self, idx: Option<usize>) -> Option<usize> {
        Self::next_with_limit(&self, idx, 6)
    }
}
