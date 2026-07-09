use crate::{
    http::{request::HttpRequest, response::HttpResponse, send_req},
    store::models::TimeId,
};

pub enum TaskType {
    HttpRequest { id: TimeId, req: HttpRequest },
}

pub enum TaskResultType {
    HttpResponse { req_id: TimeId, res: HttpResponse },
}

pub struct Task<K> {
    pub group_key: K,
    pub task: TaskType,
}

pub struct TaskResult<K> {
    pub group_key: K,
    pub result: TaskResultType,
}

type SenderTask<Key> = tokio::sync::mpsc::Sender<Task<Key>>;
type SenderResult<Key> = tokio::sync::mpsc::Sender<TaskResult<Key>>;

pub struct Tasks<Key> {
    _jh: tokio::task::JoinHandle<()>,
    tx: SenderTask<Key>,
}

impl<K: Send + 'static> Tasks<K> {
    pub fn create(rx: SenderResult<K>) -> Self {
        let (sender, mut receiver) = tokio::sync::mpsc::channel(100);

        let jh = tokio::spawn(async move {
            // TODO:  handle the abort spawn task gracefully with a map of JoinHandles
            while let Some(Task { group_key, task }) = receiver.recv().await {
                match task {
                    TaskType::HttpRequest { id, req } => {
                        let result_sender = rx.clone();
                        tokio::spawn(async move {
                            let res = send_req(req).await.unwrap();
                            result_sender
                                .send(TaskResult {
                                    group_key,
                                    result: TaskResultType::HttpResponse { req_id: id, res },
                                })
                                .await;
                        });
                    }
                }
            }
        });

        Self {
            _jh: jh,
            tx: sender,
        }
    }

    pub fn setup() -> (tokio::sync::mpsc::Receiver<TaskResult<K>>, Self) {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        (rx, Self::create(tx))
    }

    pub fn sender_for_group(&self, group_key: K) -> TaskSender<K> {
        TaskSender {
            group_key,
            sender: self.tx.clone(),
        }
    }
}

pub struct TaskSender<K> {
    group_key: K,
    sender: SenderTask<K>,
}

impl<K: Clone> TaskSender<K> {
    pub async fn send(&self, task: TaskType) {
        self.sender
            .send(Task {
                group_key: self.group_key.clone(),
                task,
            })
            .await;
    }
}
