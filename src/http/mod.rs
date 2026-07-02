use std::{env, path::PathBuf, str::FromStr, sync::OnceLock, time::Instant};

use mime::Mime;
use request::HttpRequest;
use reqwest::header::{CONTENT_TYPE, SET_COOKIE};
use response::{Cookie, HttpBodyContent, HttpResBody, HttpResponse};
use tokio::io::AsyncWriteExt;

use crate::store::models::time_as_id;

mod request;
mod response;

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

const MAX_BODY_SIZE: usize = 1024 * 1024 * 5;

pub enum Error {
    Io(std::io::Error),
}

pub fn set_http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| reqwest::Client::new())
}

enum HttpBodyWriter {
    InMemory(Vec<u8>),
    OnDisk {
        path: PathBuf,
        bytes_count: usize,
        file: tokio::fs::File,
    },
}

pub async fn send_req(req: HttpRequest) -> Option<HttpResponse> {
    let request = into_request(req).await?;

    let start = Instant::now();
    let mut response = HTTP_CLIENT.get().unwrap().execute(request).await.unwrap();
    let elapsed = start.elapsed();

    let status = response.status().as_u16();
    let version = format!("{:?}", response.version());
    let status_text = response.status().as_str().to_string();

    let mut content_type: Option<Mime> = None;
    let mut headers = Vec::new();
    let mut cookies = Vec::new();

    for (header_name, header_value) in response.headers() {
        let Ok(header_value) = header_value.to_str() else {
            println!("Could'nt parse the header_value");
            continue;
        };

        if header_name == CONTENT_TYPE {
            content_type = Mime::from_str(header_value).ok();
        }

        if header_name == SET_COOKIE {
            if let Some(cookie) = Cookie::from_str(header_value) {
                cookies.push(cookie);
            }
        }

        headers.push((header_name.to_string(), header_value.to_string()));
    }

    let mut body_writer = HttpBodyWriter::InMemory(Vec::new());
    while let Ok(chunk_opt) = response.chunk().await {
        if let Some(chunk) = chunk_opt {
            match body_writer {
                HttpBodyWriter::InMemory(ref mut content) => {
                    if content.len() + chunk.len() > MAX_BODY_SIZE {
                        let temp_file = format!(
                            "{}/kai_request_{}.tmp",
                            env::temp_dir().display(),
                            time_as_id()
                        );

                        let mut file = tokio::fs::File::create(&temp_file).await.ok()?;
                        file.write_all(content).await.ok()?;
                        file.write_all(&chunk).await.ok()?;

                        body_writer = HttpBodyWriter::OnDisk {
                            path: PathBuf::from(temp_file),
                            bytes_count: content.len() + chunk.len(),
                            file,
                        }
                    } else {
                        content.extend(chunk.to_vec());
                    }
                }
                HttpBodyWriter::OnDisk {
                    ref mut bytes_count,
                    ref mut file,
                    ..
                } => {
                    file.write_all(&chunk).await.ok()?;
                    *bytes_count += chunk.len();
                }
            }
        }
    }

    let body = match body_writer {
        HttpBodyWriter::InMemory(items) => match content_type.as_ref() {
            val if val == Some(&mime::APPLICATION_JAVASCRIPT)
                || val == Some(&mime::TEXT_JAVASCRIPT) =>
            {
                HttpResBody::Contained(HttpBodyContent::Text(
                    String::from_utf8(items).unwrap_or("Error parsing".into()),
                ))
            }
            val if val == Some(&mime::APPLICATION_JSON) => HttpResBody::Contained(
                HttpBodyContent::Text(String::from_utf8(items).unwrap_or("Error parsing".into())),
            ),
            _ => HttpResBody::Contained(HttpBodyContent::Bytes(items)),
        },
        HttpBodyWriter::OnDisk { path, mut file, .. } => {
            let _ = file.flush().await;
            HttpResBody::DiskFile(path)
        }
    };

    Some(HttpResponse {
        status,
        status_text,
        version,
        duration: elapsed,
        content_type: content_type
            .map(|m| m.essence_str().to_string())
            .unwrap_or("unknown".to_string()),
        headers,
        cookies,
        body,
    })
}

pub async fn into_request(req: HttpRequest) -> Option<reqwest::Request> {
    let client = HTTP_CLIENT.get().unwrap();
    let mut request_builder = client.request(req.method, req.url).headers(req.headers);

    match req.body {
        request::HttpBody::Text(bytes) => {
            request_builder = request_builder.body(bytes);
        }
        request::HttpBody::File(path_buf) => {
            let content = tokio::fs::read(path_buf).await.ok()?;
            request_builder = request_builder.body(reqwest::Body::from(content));
        }
        request::HttpBody::Empty => {}
    }

    request_builder.build().ok()
}
