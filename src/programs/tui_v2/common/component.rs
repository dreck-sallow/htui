pub trait Component {
    fn init(&mut self);
    fn end(mut self);
}
