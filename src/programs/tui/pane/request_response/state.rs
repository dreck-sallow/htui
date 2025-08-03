use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use tokio::{sync::oneshot, task::JoinHandle};

use crate::store::models::{SendRequest, SendRequestKey};

pub type SendRequestResponse = Arc<RwLock<SendRequest>>;

pub struct RequestResponseState {
    inner: HashMap<SendRequestKey, SendRequestResponse>,
    tasks: HashMap<SendRequestKey, RequestTask>,
}

impl RequestResponseState {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    pub fn add_response(
        &mut self,
        id: SendRequestKey,
        send_request: SendRequestResponse,
        request_task: RequestTask,
    ) {
        self.inner.insert(id.clone(), send_request);
        self.tasks.insert(id, request_task);
    }

    pub fn get(&self, id: &SendRequestKey) -> Option<SendRequestResponse> {
        self.inner.get(id).cloned()
    }

    pub fn stop(&mut self, id: &SendRequestKey) -> Option<SendRequestResponse> {
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
