use std::{collections::HashMap, str::FromStr};

use clap::Parser;
use reqwest::header::{self, HeaderName, HeaderValue};
use serde::Serialize;

mod store;

#[derive(Parser, Debug)]
#[command(name = "HTUI")]
#[command(author, version, about)]
struct Cli {
    /// Url to make the request
    #[arg(short, long)]
    url: Option<String>,

    // headers: Option<HashMap<String, String>>,
    /// Http method to execute
    method: Option<CliHttpMethod>,

    /// Headers used to make the request
    #[arg(short = 'H', long, value_delimiter = ':')]
    headers: Option<String>,

    /// Body data used to make the request
    #[arg(short = 'd', long, value_delimiter = ',')]
    body: Option<String>,
}
#[derive(clap::ValueEnum, Debug, Clone, Default, Serialize)]
#[serde(rename_all = "UPPERCASE")]
enum CliHttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
}

impl From<CliHttpMethod> for reqwest::Method {
    fn from(value: CliHttpMethod) -> Self {
        match value {
            CliHttpMethod::Get => reqwest::Method::GET,
            CliHttpMethod::Post => reqwest::Method::POST,
            CliHttpMethod::Put => reqwest::Method::PUT,
            CliHttpMethod::Delete => reqwest::Method::DELETE,
            CliHttpMethod::Patch => reqwest::Method::PATCH,
            CliHttpMethod::Head => reqwest::Method::HEAD,
        }
    }
}

// impl From<reqwest::Method> for CliHttpMethod {
//     fn from(self) -> reqwest::Method {
//         match self {
//             CliHttpMethod::Get => reqwest::Method::GET,
//             CliHttpMethod::Post => reqwest::Method::POST,
//             CliHttpMethod::Put => reqwest::Method::PUT,
//             CliHttpMethod::Delete => reqwest::Method::DELETE,
//             CliHttpMethod::Patch => reqwest::Method::PATCH,
//             CliHttpMethod::Head => reqwest::Method::HEAD,
//         }
//     }
// }
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // 1. url && not method -> Make an GET request to url defined
    // 2. url && method -> Make an [method] request useing th url defined

    if let Some(url) = cli.url {
        let cli_method = cli.method.unwrap_or(CliHttpMethod::Get);
        let headers = match cli.headers {
            Some(ref raw_headers) => {
                let parsed: HashMap<String, String> = serde_json::from_str(raw_headers).unwrap(); //FIXME: fix the header parsing

                let mut header_map = header::HeaderMap::new();
                for (key, val) in parsed {
                    let header_parsed = HeaderName::from_str(&key.to_lowercase()).unwrap(); //FIXME: fix the error parsing
                    let header_value_parsed = HeaderValue::from_str(&val.to_lowercase()).unwrap(); //FIXME: fix the error parsing
                    header_map.insert(header_parsed, header_value_parsed);
                }

                header_map
            }
            None => header::HeaderMap::default(),
        };

        let body = match cli.body {
            Some(raw_body) => reqwest::Body::from(raw_body),
            None => reqwest::Body::default(),
        };

        let client = reqwest::Client::new()
            .request(cli_method.into(), url)
            .headers(headers)
            .body(body);
        let res = client.send().await.unwrap();
        return println!(
            "Make the request with response: {}",
            res.text().await.unwrap()
        );
    }
    println!("Launch the TUI application");
}
