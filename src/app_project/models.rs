use std::{
    collections::HashMap,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{
    de::{self, Visitor},
    ser::SerializeMap,
    Deserialize, Serialize,
};
use tempfile::TempDir;

pub fn time_as_id() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProjectModel {
    id: String,
    pub name: String,
    pub collections: Vec<CollectionsModel>,
}

impl ProjectModel {
    pub fn new(name: String) -> Self {
        Self {
            id: time_as_id(),
            name,
            collections: Vec::new(),
        }
    }

    pub fn from_parts(id: String, name: String, collections: Vec<CollectionsModel>) -> Self {
        Self {
            id,
            name,
            collections,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Default for ProjectModel {
    fn default() -> Self {
        let id = time_as_id();
        Self {
            id: id.clone(),
            name: id,
            collections: Vec::new(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CollectionsModel {
    id: String,
    pub name: String,
    pub requests: Vec<RequestModel>,
}

impl CollectionsModel {
    pub fn new(name: String) -> Self {
        Self {
            id: time_as_id(),
            name,
            requests: Vec::new(),
        }
    }

    pub fn from_parts(id: String, name: String, requests: Vec<RequestModel>) -> Self {
        Self { id, name, requests }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn requests(&self) -> &[RequestModel] {
        &self.requests
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct KeyValueParam {
    /// Property for indicate if its value will applied to the request
    pub enable: bool,
    pub key: String,
    pub value: String,
}

impl KeyValueParam {
    pub fn new(key: String, value: String) -> Self {
        Self {
            enable: true,
            key,
            value,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RequestModel {
    id: String,
    name: String,
    url: String,
    headers: HashMap<String, String>,
    pub params: Vec<KeyValueParam>,
    method: HttpMethod,
    body: BodyContent,
}

impl RequestModel {
    pub fn new(name: String) -> Self {
        Self {
            id: time_as_id(),
            name,
            url: String::from("https://"),
            headers: HashMap::default(),
            params: Vec::new(),
            method: HttpMethod::Get,
            body: BodyContent::Empty,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }

    pub fn headers_map(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn headers(&self) -> Vec<(&str, &str)> {
        let mut list = Vec::new();

        for (k, v) in &self.headers {
            list.push((k.as_str(), v.as_str()));
        }

        list
    }

    pub fn set_headers(&mut self, headers: HashMap<String, String>) {
        self.headers = headers;
    }

    pub fn body(&self) -> &BodyContent {
        &self.body
    }

    pub fn set_body(&mut self, body: BodyContent) {
        self.body = body;
    }

    pub fn method(&self) -> HttpMethod {
        self.method
    }

    pub fn set_method(&mut self, method: HttpMethod) {
        self.method = method;
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

// Define the http model type

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HttpMethod {
    Options,
    Get,
    Post,
    Put,
    Delete,
    Head,
    Patch,
}

impl AsRef<str> for HttpMethod {
    fn as_ref(&self) -> &'static str {
        match self {
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Patch => "PATCH",
        }
    }
}

impl Serialize for HttpMethod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let txt: &str = self.as_ref();
        serializer.serialize_str(txt)
    }
}

struct HttpMethodVisitor;

impl Visitor<'_> for HttpMethodVisitor {
    type Value = HttpMethod;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "An valid defined Http Method ")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match HttpMethod::try_from(v) {
            Ok(method) => Ok(method),
            Err(_) => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(v),
                &self,
            )),
        }
    }
}

impl<'de> Deserialize<'de> for HttpMethod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(HttpMethodVisitor)
    }
}

// impl Into<&str> for &HttpMethod {
//     fn into(self) -> &'static str {
//         match self {
//             HttpMethod::Options => "OPTIONS",
//             HttpMethod::Get => "GET",
//             HttpMethod::Post => "POST",
//             HttpMethod::Put => "PUT",
//             HttpMethod::Delete => "DELETE",
//             HttpMethod::Head => "HEAD",
//             HttpMethod::Patch => "PATCH",
//         }
//     }
// }

/// Structure only used for parse a str to httpMethod;
pub struct ParseErrorHttpMethod;

impl TryFrom<&str> for HttpMethod {
    type Error = ParseErrorHttpMethod;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "OPTIONS" => Ok(Self::Options),
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "HEAD" => Ok(Self::Head),
            "PATCH" => Ok(Self::Patch),
            _ => Err(ParseErrorHttpMethod),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BodyContent {
    Empty,
    File(PathBuf),
    Form(HashMap<String, String>), // FIXME: use another value for the hashmap
    Text(String),
}

impl BodyContent {
    pub fn as_tag(&self) -> &str {
        match self {
            BodyContent::Empty => "Empty",
            BodyContent::File(_) => "File",
            BodyContent::Form(_) => "Form",
            BodyContent::Text(_) => "Text",
        }
    }
}

impl Serialize for BodyContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;

        match self {
            BodyContent::Empty => map.serialize_entry("empty", &())?,
            BodyContent::Text(str) => map.serialize_entry("text", str)?,
            BodyContent::File(path_buf) => {
                map.serialize_entry("file", path_buf.as_os_str().to_str().unwrap())?
            }
            BodyContent::Form(form) => map.serialize_entry("form", form)?,
        }

        map.end()
    }
}

pub struct BodyVisitor;

impl<'de> Visitor<'de> for BodyVisitor {
    type Value = BodyContent;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "An valid defined body content ")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let Some((key, val)) = map.next_entry::<String, serde_json::Value>()? else {
            return Ok(BodyContent::Empty);
        };

        match key.as_str() {
            "text" => {
                let text = serde_json::from_value::<String>(val).map_err(de::Error::custom)?;
                Ok(BodyContent::Text(text))
            }
            "file" => {
                let file_path = serde_json::from_value::<String>(val).map_err(de::Error::custom)?;
                Ok(BodyContent::File(PathBuf::from(file_path)))
            }
            "form" => {
                let form = serde_json::from_value::<HashMap<String, String>>(val)
                    .map_err(de::Error::custom)?;
                Ok(BodyContent::Form(form))
            }
            "empty" => Ok(BodyContent::Empty),
            other => Err(de::Error::unknown_field(
                other,
                &["text", "file", "form", "empty"],
            )),
        }
    }
}

impl<'de> Deserialize<'de> for BodyContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(BodyVisitor)
    }
}

pub struct ResponseModel {
    pub duration: Duration,
    pub status: u16,
    /// Body types, storing bytes from (Bytes struct)
    pub body: Vec<u8>,
    pub file_path: ResponseFilePath,
    pub headers: Vec<(String, String)>,
}

pub enum ResponseFilePath {
    Temp(tempfile::TempPath),
    Saved(PathBuf),
    Null,
}

/// Used for track the request that is currently tracked on async tasks
/// Is using the index for collection & request
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct SendRequestKey {
    pub collection_id: String,
    pub request_id: String,
}

impl From<(String, String)> for SendRequestKey {
    fn from(value: (String, String)) -> Self {
        Self {
            collection_id: value.0,
            request_id: value.1,
        }
    }
}

pub enum SendRequest {
    Pending,
    Finish(ResponseModel),
}
