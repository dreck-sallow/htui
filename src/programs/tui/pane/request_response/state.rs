use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use tokio::{sync::oneshot, task::JoinHandle};

use crate::store::models::{SendRequest, SendRequestId};

pub type SendRequestResponse = Arc<RwLock<SendRequest>>;

pub struct RequestResponseState {
    inner: HashMap<SendRequestId, SendRequestResponse>,
    tasks: HashMap<SendRequestId, RequestTask>,
    current_response: Option<SendRequestId>,
}

impl RequestResponseState {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            tasks: HashMap::new(),
            current_response: None,
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

    pub fn get(&self, id: &SendRequestId) -> Option<SendRequestResponse> {
        self.inner.get(id).cloned()
    }

    pub fn current_response(&self) -> Option<&SendRequestId> {
        self.current_response.as_ref()
    }

    pub fn set_current_response(&mut self, request_id: Option<SendRequestId>) {
        self.current_response = request_id;
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
