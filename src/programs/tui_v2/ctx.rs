use super::{app::TaskGroupKey, events::DrawSignal, task::TaskSender};

pub struct InitialCtx {
    pub draw_signal: DrawSignal,
    pub task_sender: TaskSender<TaskGroupKey>,
}
