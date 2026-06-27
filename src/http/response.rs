use std::time::Duration;

pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub version: String,
    pub duration: Duration,
    pub content_type: String,
    pub headers: Vec<(String, String)>,
    pub cookies: Vec<Cookie>,
    // pub body: Body,
}

pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub expires: Option<String>,
    pub max_age: Option<String>,
    pub path: Option<String>,
    pub http_only: Option<bool>,
    pub partitioned: Option<bool>,
    pub secure: Option<bool>,
    pub same_site: Option<String>,
}
