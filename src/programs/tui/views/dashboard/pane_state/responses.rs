use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use tokio::{sync::oneshot, task::JoinHandle};

use crate::store::models::{SendRequest, SendRequestId};

pub type SendRequestResponse = Arc<RwLock<SendRequest>>;

pub struct Responses {
    inner: HashMap<SendRequestId, SendRequestResponse>,
    tasks: HashMap<SendRequestId, RequestTask>,
}

impl Responses {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    pub fn add(
        &mut self,
        id: SendRequestId,
        send_request: SendRequestResponse,
        request_task: RequestTask,
    ) {
        self.inner.insert(id.clone(), send_request);
        self.tasks.insert(id, request_task);
    }

    pub fn contains(&mut self, id: SendRequestId) {
        self.inner.contains_key(&id);
    }

    pub fn get(&self, id: &SendRequestId) -> Option<SendRequestResponse> {
        self.inner.get(id).cloned()
    }

    pub fn stop(&mut self, id: &SendRequestId) -> Option<SendRequestResponse> {
        match self.inner.remove(id) {
            Some(req) => {
                if let SendRequest::Pending = &*req.read().unwrap() {
                    self.tasks.remove(id).unwrap().cancel();
                }
                Some(req)
            }
            None => None,
        }
    }
}

pub struct RequestTask {
    cancel_sender: oneshot::Sender<()>,
    task: JoinHandle<()>,
}

impl RequestTask {
    pub fn new(cancel_sender: oneshot::Sender<()>, task: JoinHandle<()>) -> Self {
        Self {
            cancel_sender,
            task: task,
        }
    }

    pub fn cancel(self) {
        // Not handle the error, so use the abort method
        let _ = self.cancel_sender.send(());
        self.task.abort();
    }
}
