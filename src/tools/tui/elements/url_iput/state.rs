use std::fmt::Display;

#[derive(Default)]
pub enum UrlMethod {
    #[default]
    Get,
    Post,
    Delete,
    Put,
}

impl Display for UrlMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            UrlMethod::Get => "GET",
            UrlMethod::Post => "POST",
            UrlMethod::Delete => "DELETE",
            UrlMethod::Put => "PUT",
        };

        write!(f, "{str}")
    }
}

#[derive(Default)]
pub struct UrlInputState {
    method: UrlMethod,
    url: String,
}

impl UrlInputState {
    pub fn new(method: UrlMethod, url: &str) -> Self {
        Self {
            method,
            url: url.to_string(),
        }
    }

    pub fn set_method(&mut self, method: UrlMethod) {
        self.method = method;
    }

    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }

    pub fn method(&self) -> &UrlMethod {
        &self.method
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}
