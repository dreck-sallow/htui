#[derive(Default)]
pub struct UrlInputState {
    url: String,
}

impl UrlInputState {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }

    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}
