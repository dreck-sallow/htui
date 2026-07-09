use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use std::path::{Path, PathBuf};

pub struct HttpRequest {
    pub(crate) method: reqwest::Method,
    pub(crate) url: reqwest::Url,
    pub(crate) headers: HeaderMap,
    pub(crate) body: HttpBody,
}

impl HttpRequest {
    pub fn new(method: reqwest::Method, url: reqwest::Url) -> Self {
        Self {
            method,
            url,
            headers: reqwest::header::HeaderMap::new(),
            body: HttpBody::Empty,
        }
    }

    pub fn add_header(&mut self, name: HeaderName, value: HeaderValue) {
        self.headers.insert(name, value);
    }

    fn set_content_type(&mut self, content_type: HeaderValue) {
        if !self.headers.contains_key(header::CONTENT_TYPE) {
            self.headers.insert(header::CONTENT_TYPE, content_type);
        }
    }

    pub fn set_body_text(&mut self, txt: Vec<u8>) {
        self.body = HttpBody::Text(txt);
        self.set_content_type("text/plain".parse().unwrap());
    }

    pub fn set_body_file(&mut self, path: PathBuf) {
        if !self.headers.contains_key(header::CONTENT_TYPE) {
            if let Some(content_type) = content_type_from_path(&path) {
                self.headers.insert(header::CONTENT_TYPE, content_type);
            }
        }
        self.body = HttpBody::File(path);
    }
}

pub enum HttpBody {
    Text(Vec<u8>),
    File(PathBuf),
    Empty,
}

pub fn content_type_from_path<P: AsRef<Path>>(path: P) -> Option<HeaderValue> {
    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    HeaderValue::from_str(mime.essence_str()).ok()
}
