use std::{env, path::PathBuf, str::FromStr, sync::OnceLock, time::Instant};

use mime::Mime;
use request::HttpRequest;
use reqwest::header::{CONTENT_TYPE, SET_COOKIE};
use response::{Cookie, HttpBodyContent, HttpResBody, HttpResponse};
use tokio::io::AsyncWriteExt;

use crate::store::models::time_as_id;

pub mod request;
pub mod response;

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

const MAX_BODY_SIZE: usize = 1024 * 1024 * 5;

pub struct Error {
    pub kind: ErrorKind,
    pub title: String,
    pub description: String,
}

pub enum ErrorKind {
    FileEror,
    BuildRequest,
    SendReq,
}

pub fn set_http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| reqwest::Client::new())
}

pub fn get_http_client() -> &'static reqwest::Client {
    HTTP_CLIENT
        .get()
        .expect("Set the HTTP client before to get the instance")
}

enum HttpBodyWriter {
    InMemory(Vec<u8>),
    OnDisk {
        path: PathBuf,
        bytes_count: usize,
        file: tokio::io::BufWriter<tokio::fs::File>,
    },
}

// pub enum HttpResult {
//     Response(HttpResponse),
//     Err(Error),
// }

pub async fn send_req(req: HttpRequest) -> Result<HttpResponse, Error> {
    let request = into_request(req).await?;

    let start = Instant::now();
    let mut response = HTTP_CLIENT.get().unwrap().execute(request).await.unwrap();
    let elapsed = start.elapsed();

    let status = response.status().as_u16();
    let version = format!("{:?}", response.version());
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("unknown")
        .to_string();

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

    let mut body_writer = HttpBodyWriter::InMemory(Vec::with_capacity(1024 * 5));
    while let Ok(Some(chunk)) = response.chunk().await {
        match body_writer {
            HttpBodyWriter::InMemory(ref mut content) => {
                if content.len() + chunk.len() > MAX_BODY_SIZE {
                    let temp_file = format!(
                        "{}/kai_request_{}.tmp",
                        env::temp_dir().display(),
                        time_as_id()
                    );

                    let mut file = tokio::io::BufWriter::new(
                        tokio::fs::File::create(&temp_file)
                            .await
                            .map_err(|e| Error {
                                kind: ErrorKind::FileEror,
                                title: "Failed to create file for save request response".into(),
                                description: e.to_string(),
                            })?,
                    );

                    file.write_all(content).await.map_err(|e| Error {
                        kind: ErrorKind::FileEror,
                        title: "Failed to write file".into(),
                        description: e.to_string(),
                    })?;
                    file.write_all(&chunk).await.map_err(|e| Error {
                        kind: ErrorKind::FileEror,
                        title: "Failed to write file".into(),
                        description: e.to_string(),
                    })?;

                    body_writer = HttpBodyWriter::OnDisk {
                        path: PathBuf::from(temp_file),
                        bytes_count: content.len() + chunk.len(),
                        file,
                    }
                } else {
                    content.extend_from_slice(&chunk);
                }
            }
            HttpBodyWriter::OnDisk {
                ref mut bytes_count,
                ref mut file,
                ..
            } => {
                file.write_all(&chunk).await.map_err(|e| Error {
                    kind: ErrorKind::FileEror,
                    title: "Failed to write file".into(),
                    description: e.to_string(),
                })?;
                *bytes_count += chunk.len();
            }
        }
    }

    let body_size = match body_writer {
        HttpBodyWriter::InMemory(ref items) => items.len(),
        HttpBodyWriter::OnDisk { bytes_count, .. } => bytes_count,
    };

    let body = match body_writer {
        HttpBodyWriter::InMemory(items) => match content_type.as_ref() {
            val if val == Some(&mime::APPLICATION_JAVASCRIPT)
                || val == Some(&mime::TEXT_JAVASCRIPT) =>
            {
                HttpResBody::Contained(HttpBodyContent::Text(
                    String::from_utf8(items).unwrap_or("Error parsing".into()),
                ))
            }
            val if val == Some(&mime::APPLICATION_JSON) => {
                HttpResBody::Contained(HttpBodyContent::Text(
                    serde_json::to_string_pretty(&items).unwrap_or("Error parsing".into()),
                ))
            }
            _ => HttpResBody::Contained(HttpBodyContent::Text(
                String::from_utf8(items).unwrap_or("Parsing error".into()),
            )),
            // _ => HttpResBody::Contained(HttpBodyContent::Bytes(items)),
        },
        HttpBodyWriter::OnDisk { path, mut file, .. } => {
            let _ = file.flush().await;
            HttpResBody::DiskFile(path)
        }
    };

    Ok(HttpResponse {
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
        body_bytes: body_size,
    })
}

async fn into_request(req: HttpRequest) -> Result<reqwest::Request, Error> {
    let client = HTTP_CLIENT.get().unwrap();
    let mut request_builder = client.request(req.method, req.url).headers(req.headers);

    match req.body {
        request::HttpBody::Text(bytes) => {
            request_builder = request_builder.body(bytes);
        }
        request::HttpBody::File(path_buf) => {
            let content = tokio::fs::read(path_buf).await.map_err(|e| Error {
                kind: ErrorKind::FileEror,
                title: "Failed to read the path content".into(),
                description: e.to_string(),
            })?;
            request_builder = request_builder.body(reqwest::Body::from(content));
        }
        request::HttpBody::Empty => {}
    }

    let req = request_builder.build().map_err(|e| Error {
        kind: ErrorKind::BuildRequest,
        title: "Failed to build request".into(),
        description: e.to_string(),
    })?;

    Ok(req)
}
