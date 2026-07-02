use std::{path::PathBuf, time::Duration};

pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub version: String,
    pub duration: Duration,
    pub content_type: String,
    pub headers: Vec<(String, String)>,
    pub cookies: Vec<Cookie>,
    pub body: HttpResBody,
}

pub enum HttpResBody {
    Contained(HttpBodyContent),
    DiskFile(PathBuf),
}

pub enum HttpBodyContent {
    Text(String),
    Bytes(Vec<u8>),
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

impl Cookie {
    pub fn from_str(txt: &str) -> Option<Self> {
        let mut parts = txt
            .split(";")
            .filter_map(|st| st.split_once('=').or(Some((st, st))));

        let (name, value) = {
            let cookie = parts.next()?;
            (cookie.0.to_string(), cookie.1.to_string())
        };

        let mut cookie = Self {
            name,
            value,
            domain: None,
            expires: None,
            max_age: None,
            path: None,
            http_only: None,
            partitioned: None,
            secure: None,
            same_site: None,
        };

        for (key, value) in parts {
            let key = key.trim();

            if key.eq_ignore_ascii_case("Domain") {
                cookie.domain = Some(value.to_string());
            } else if key.eq_ignore_ascii_case("Expires") {
                cookie.expires = Some(value.to_string());
            } else if key.eq_ignore_ascii_case("Max-Age") {
                cookie.max_age = Some(value.to_string());
            } else if key.eq_ignore_ascii_case("Path") {
                cookie.path = Some(value.to_string());
            } else if key.eq_ignore_ascii_case("SameSite") {
                cookie.same_site = Some(value.to_string());
            } else if key.eq_ignore_ascii_case("Secure") {
                cookie.secure = Some(true);
            } else if key.eq_ignore_ascii_case("HttpOnly") {
                cookie.http_only = Some(true);
            } else if key.eq_ignore_ascii_case("Partitioned") {
                cookie.partitioned = Some(true);
            }
        }

        Some(cookie)
    }
}
