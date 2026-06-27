use std::{sync::OnceLock, time::Instant};

use request::HttpRequest;
use response::HttpResponse;

mod request;
mod response;

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub enum Error {
    Io(std::io::Error),
}

pub fn set_http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| reqwest::Client::new())
}

pub async fn send_req(req: HttpRequest) -> Option<HttpResponse> {
    let request = into_request(req).await?;

    let start = Instant::now();
    let response = HTTP_CLIENT.get().unwrap().execute(request).await.unwrap();

    let elapsed = start.elapsed();
    let status = response.status().as_u16();
    let version = format!("{:?}", response.version());
    let status_text = response.status().as_str().to_string();

    // TODO: read the body, and if we reach out the limit 50MB, then redirect to disk file
    // response.chunk();

    Some(HttpResponse {
        status,
        status_text,
        version,
        duration: elapsed,
        content_type: "".into(),
        headers: Vec::new(),
        cookies: Vec::new(),
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
