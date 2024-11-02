use reqwest::{header, Body};
use std::{collections::HashMap, io, str::FromStr};

use crate::Cli;

#[derive(Debug)]
pub enum CliAppError {
    ParseCliArgs,
}

pub struct CliAppOptions {
    url: String, // TODO: change to uri or something similar,
    method: reqwest::Method,
    headers: HashMap<String, String>,
    body: String, // TODO: check if its necesary pass as string
}

impl TryFrom<Cli> for CliAppOptions {
    type Error = CliAppError;
    fn try_from(cli: Cli) -> Result<Self, Self::Error> {
        let url = cli.url.ok_or(CliAppError::ParseCliArgs)?;
        let method = cli.method.unwrap_or_default();

        let headers = match cli.headers {
            Some(st) => {
                let pairs: Vec<(String, String)> = st
                    .split(',')
                    .map(|range| {
                        let splitted: Vec<&str> = range.split(':').collect();
                        let o = (
                            splitted.first().map(ToString::to_string),
                            splitted.get(1).map(ToString::to_string),
                        );
                        o
                    })
                    .filter_map(|range| match range {
                        (Some(key), Some(val)) => Some((key, val)),
                        _ => None,
                    })
                    .collect();

                HashMap::from_iter(pairs)
            }
            None => HashMap::default(),
        };
        println!("headers: {:?}", headers);

        Ok(Self {
            url,
            method: method.into(),
            headers,
            body: cli.body.unwrap_or_default(),
        })
    }
}

pub async fn run_app(options: CliAppOptions) -> io::Result<()> {
    // build the headers from the options
    let mut headers = header::HeaderMap::default();
    options.headers.iter().for_each(|(k, v)| {
        if let (Ok(header_name), Ok(hader_value)) = (
            header::HeaderName::from_str(k),
            header::HeaderValue::from_str(v),
        ) {
            headers.insert(header_name, hader_value);
        }
    });

    // build the body from the options

    let body = Body::from(options.body);

    let client = reqwest::Client::new()
        .request(options.method, options.url)
        .headers(headers)
        .body(body);

    let res = client.send().await.unwrap();
    println!("response: {}", res.text().await.unwrap());
    Ok(())
}
