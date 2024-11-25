use std::str::FromStr;

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Body, Client, Method,
};

use crate::cmd_args::CmdArgs;

pub struct CliParams {
    url: String,
    method: Method,
    headers: HeaderMap,
    body: Body,
}

pub async fn run(options: CliParams) {
    let response = Client::new()
        .request(options.method, options.url)
        .headers(options.headers)
        .body(options.body)
        .send()
        .await
        .unwrap();

    let text = response.text().await.unwrap();

    println!("response: {text}");
}

#[derive(Debug)]
pub enum CliToolError {
    MissingUrl,
}
// Pass from cmd_args to CliParams

impl TryFrom<CmdArgs> for CliParams {
    type Error = CliToolError;

    fn try_from(value: CmdArgs) -> Result<Self, Self::Error> {
        let url = match value.url {
            Some(url) => url,
            None => return Err(CliToolError::MissingUrl),
        };

        let method = value
            .method
            .map(|method| match method {
                crate::cmd_args::CmdHttpMethod::Get => Method::GET,
                crate::cmd_args::CmdHttpMethod::Post => Method::POST,
                crate::cmd_args::CmdHttpMethod::Put => Method::PUT,
                crate::cmd_args::CmdHttpMethod::Patch => Method::PATCH,
                crate::cmd_args::CmdHttpMethod::Delete => Method::DELETE,
            })
            .unwrap_or(Method::GET);

        let headers = match value.headers {
            Some(headers) => {
                // TODO: define a better way for manage raw string header, currently use ',' separator
                let values: Vec<(&str, &str)> = headers
                    .split(',')
                    .filter_map(|str| {
                        let mut str_entry = str.split(':');
                        let key = str_entry.nth(0).map(|st| st.trim());
                        let value = str_entry.nth(1).map(|st| st.trim());
                        match (key, value) {
                            (Some(key), Some(val)) => Some((key, val)),
                            _ => None,
                        }
                    })
                    .collect();

                let mut headers_map = HeaderMap::default();

                for (key, val) in values {
                    let header_name = HeaderName::from_str(&key.to_lowercase()).unwrap();
                    let header_value = HeaderValue::from_str(&val.to_lowercase()).unwrap();
                    headers_map.insert(header_name, header_value);
                }

                headers_map
            }
            None => HeaderMap::default(),
        };

        Ok(Self {
            url,
            method,
            headers,
            body: value.body.unwrap_or_default().into(),
        })
    }
}
