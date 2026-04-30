use std::time::Duration;

use tokio::sync::mpsc;

use crate::store::models::TimeId;

pub type EventReceiver = mpsc::Receiver<AppEvent>;

pub struct AppEvent {
    pub pane_id: TimeId,
    pub event: Event,
}

pub enum Event {
    Response {
        req_id: TimeId,
        response: ReqResponse,
    },
}

pub type TaskSender = mpsc::Sender<AppTask>;
pub type TaskReceiver = mpsc::Receiver<AppTask>;

pub struct AppTask {
    pub pane_id: TimeId,
    pub task: Task,
}

impl AppTask {
    pub fn for_request(pane_id: TimeId, request_id: TimeId, req: reqwest::RequestBuilder) -> Self {
        Self {
            pane_id,
            task: Task::Request {
                request_id,
                request: req,
            },
        }
    }
}

pub enum Task {
    Request {
        request_id: TimeId,
        request: reqwest::RequestBuilder,
    },
}

pub fn create_background_tasks() -> (TaskSender, BackgroundTasks) {
    let (send_tasks, rx) = mpsc::channel(100);
    (send_tasks, BackgroundTasks::new(rx))
}

pub struct BackgroundTasks {
    _tx: tokio::task::JoinHandle<()>,
    receiver: EventReceiver,
}

impl BackgroundTasks {
    pub fn new(mut rx: TaskReceiver) -> Self {
        let (sender, receiver) = mpsc::channel(100);

        let jh = tokio::spawn(async move {
            // TODO:  handle the abort spawn task gracefully with a map of JoinHandles
            while let Some(AppTask { pane_id, task }) = rx.recv().await {
                match task {
                    Task::Request {
                        request_id,
                        request,
                    } => {
                        let _tx = sender.clone();
                        tokio::spawn(async move {
                            let res = process_request(request).await;
                            let _ = _tx
                                .send(AppEvent {
                                    pane_id,
                                    event: Event::Response {
                                        req_id: request_id,
                                        response: res,
                                    },
                                })
                                .await;
                        });
                    }
                }
            }
        });

        Self { _tx: jh, receiver }
    }

    pub fn stop(&self) {
        self._tx.abort();
    }

    pub async fn next_event(&mut self) -> Option<AppEvent> {
        self.receiver.recv().await
    }
}

pub enum ReqResponse {
    Err(String),
    Sucess(Response),
}

pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub version: String,
    pub duration: Duration,
    pub headers: Vec<(String, String)>,
}

pub async fn process_request(req: reqwest::RequestBuilder) -> ReqResponse {
    let timer = tokio::time::Instant::now();
    let result = req.send().await;
    let duration = timer.elapsed();

    let res = match result {
        Ok(r) => r,
        Err(e) => {
            return ReqResponse::Err(e.to_string());
        }
    };

    let mut headers = Vec::new();

    for (name, value) in res.headers() {
        // NOTE: support non-ascii text?
        let Ok(value) = value.to_str() else {
            continue;
        };
        headers.push((name.as_str().to_string(), value.to_string()));
    }

    ReqResponse::Sucess(Response {
        status: res.status().as_u16(),
        status_text: res.status().as_str().to_string(),
        version: format!("{:?}", res.version()),
        duration,
        headers,
        // body: super::state::ResponseBody::Empty,
        // size_bytes: res.bytes().await.map(|b| b.len()).unwrap_or(0),
    })
}
