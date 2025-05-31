pub trait WithHistory {
    fn undo(&mut self);
    fn redo(&mut self);
}
