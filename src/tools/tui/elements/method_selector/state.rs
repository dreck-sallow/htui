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

impl From<&str> for UrlMethod {
    fn from(value: &str) -> Self {
        match value {
            "POST" => UrlMethod::Post,
            "DELETE" => UrlMethod::Delete,
            "PUT" => UrlMethod::Put,
            _ => UrlMethod::Get,
        }
    }
}

#[derive(Default)]
pub struct MethodSelectorState {
    method_type: UrlMethod,
}

impl MethodSelectorState {
    pub fn set_method(&mut self, method: UrlMethod) {
        self.method_type = method;
    }

    pub fn method(&self) -> &UrlMethod {
        &self.method_type
    }
}
