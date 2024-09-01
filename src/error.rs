#[derive(Debug)]
pub enum AppError {
    RequestParsing(&'static str),
}
