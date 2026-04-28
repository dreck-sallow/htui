use std::time::Duration;

use tokio::sync::mpsc;

use crate::store::models::TimeId;

// pub type EventSender = mpsc::Sender<AppEvent>;
pub type EventReceiver = mpsc::Receiver<AppEvent>;

pub enum AppEvent {
    Response(ReqResponse),
}

pub type TaskSender = mpsc::Sender<AppTask>;
pub type TaskReceiver = mpsc::Receiver<AppTask>;
pub enum AppTask {
    Request(TimeId, reqwest::RequestBuilder),
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
            while let Some(ev) = rx.recv().await {
                match ev {
                    AppTask::Request(req_id, builder) => {
                        let _tx = sender.clone();
                        tokio::spawn(async move {
                            let res = process_request(req_id, builder).await;
                            let _ = _tx.send(AppEvent::Response(res)).await;
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
    // Err { id: TimeId, msg: String },
    Err(TimeId, String),
    // Sucess { id: TimeId, response: Response },
    Sucess(TimeId, Response),
}

pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub version: String,
    pub duration: Duration,
}

pub async fn process_request(req_id: TimeId, req: reqwest::RequestBuilder) -> ReqResponse {
    let timer = tokio::time::Instant::now();
    let result = req.send().await;
    let duration = timer.elapsed();

    let res = match result {
        Ok(r) => r,
        Err(e) => {
            return ReqResponse::Err(req_id, e.to_string());
        }
    };

    ReqResponse::Sucess(
        req_id,
        Response {
            status: res.status().as_u16(),
            status_text: res.status().as_str().to_string(),
            version: format!("{:?}", res.version()),
            duration,
            // headers: headers,
            // body: super::state::ResponseBody::Empty,
            // size_bytes: res.bytes().await.map(|b| b.len()).unwrap_or(0),
        },
    )
}
