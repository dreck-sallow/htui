use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use encoding_rs::{Encoding, UTF_8};
use ratatui::layout::Rect;
use reqwest::{header::CONTENT_TYPE, ClientBuilder, RequestBuilder, Response, Url};
use tokio::time::Instant;

use crate::{
    app_project::models::{self, RequestModel, ResponseModel, SendRequest, SendRequestKey},
    programs::tui::{event_handler::EventSender, pane::text_editor::TextEditor},
};

use super::{
    binary_viewer::BinaryViewer,
    body_viewer::BodyContentView,
    state::{RequestTask, SendRequestResponse},
    ResponseContent,
};

pub fn request_model_to_state(
    response_model: &ResponseModel,
    response_content: &mut ResponseContent,
    view_area: Rect,
) {
    response_content
        .headers_table
        .replace(response_model.headers.as_slice());

    let content_type = response_model
        .headers
        .iter()
        .find(|(k, _v)| k == CONTENT_TYPE.as_str())
        .and_then(|(_k, v)| v.parse::<mime::Mime>().ok());

    if let Some(mime_type) = content_type {
        let is_text_based = match (mime_type.type_(), mime_type.subtype()) {
            (mime::TEXT, _) => true,
            (mime::APPLICATION, sub) => {
                matches!(
                    sub.as_str(),
                    "json" | "xml" | "xhtml+xml" | "x-www-form-urlencoded"
                )
            }
            _ => false,
        };

        if is_text_based {
            let encoding_name = mime_type
                .get_param("charset")
                .map(|charset| charset.as_str())
                .unwrap_or("utf-8");

            let encoding = Encoding::for_label(encoding_name.as_bytes()).unwrap_or(UTF_8);
            let (text, _, _) = encoding.decode(&response_model.body);
            let mut text_editor = TextEditor::new(false);

            text_editor.insert_str(text.as_ref());

            response_content.body_viewer = BodyContentView::text(text_editor, view_area);
            return;
        }
    }

    let dump_viewer = BinaryViewer::from_bytes(&response_model.body, view_area);
    response_content.body_viewer = BodyContentView::Binary(dump_viewer);
}

pub struct SendRequestContext {
    pub(crate) request_response: SendRequestResponse,
    pub(crate) current_key: SendRequestKey,
    pub(crate) response_content: Arc<RwLock<ResponseContent>>,
}

pub fn send_request(
    request_build: RequestBuilder,
    context: SendRequestContext,
    sender: EventSender,
    view_area: Rect,
) -> RequestTask {
    // Signal channel to finish the async task
    let (tx, tr) = tokio::sync::oneshot::channel();

    let task = tokio::spawn(async move {
        let timer = Instant::now();

        tokio::select! {
            http_response = request_build.send() => {
                match http_response {
                    Ok(response) => {
                        let response_model = into_response_model(response, timer.elapsed()).await;
                        *context.request_response.write().unwrap() = SendRequest::Finish(response_model);

                        if context.response_content.read().unwrap().request_key.as_ref().map(|key| * key != context.current_key).unwrap_or(false) {
                            // So the current visual UI request is diferent from the local request executing
                            return;
                        }

                        let mut locked = context.response_content.write().unwrap();
                        // let response_model = &*context.request_response.read().unwrap();

                        if let SendRequest::Finish(response_model) = &*context.request_response.read().unwrap() {
                            request_model_to_state(response_model, &mut *locked, view_area);
                        }
                    }
                    Err(_err) => {

                    }
                }

            }
            _ = tr => {

            }
        }

        let _ = sender
            .send(crate::programs::tui::event_handler::AppMessage::Draw)
            .await;
    });

    RequestTask::new(tx, task)
}

async fn into_response_model(http_response: Response, duration: Duration) -> ResponseModel {
    let key_value_headers: Vec<(String, String)> = http_response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap().into()))
        .collect();
    let response_status = http_response.status().as_u16();
    let response_body = http_response.bytes().await.unwrap();

    ResponseModel {
        duration,
        status: response_status,
        body: response_body.to_vec(),
        headers: key_value_headers.clone(),
    }
}

pub fn request_into_builder(req: &RequestModel) -> RequestBuilder {
    let url = Url::parse(req.url()).unwrap();
    let method = match req.method() {
        models::HttpMethod::Options => reqwest::Method::OPTIONS,
        models::HttpMethod::Get => reqwest::Method::GET,
        models::HttpMethod::Post => reqwest::Method::POST,
        models::HttpMethod::Put => reqwest::Method::PUT,
        models::HttpMethod::Delete => reqwest::Method::DELETE,
        models::HttpMethod::Head => reqwest::Method::HEAD,
        models::HttpMethod::Patch => reqwest::Method::PATCH,
    };

    let mut client_builder = ClientBuilder::new()
        .referer(false)
        .build()
        .unwrap()
        .request(method, url);

    for (key, value) in req.headers_map() {
        client_builder = client_builder.header(key, value);
    }

    client_builder
}
