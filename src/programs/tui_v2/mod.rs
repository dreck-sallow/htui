use std::io;

mod app;
mod common;
mod config;
mod events;

#[derive(Debug)]
pub enum TuiError {
    Io(io::Error),
    Config(toml::de::Error),
}

impl From<io::Error> for TuiError {
    fn from(value: io::Error) -> Self {
        TuiError::Io(value)
    }
}

impl From<toml::de::Error> for TuiError {
    fn from(value: toml::de::Error) -> Self {
        TuiError::Config(value)
    }
}

pub type TuiResult<T> = Result<T, TuiError>;

// pub async fn run(project_name: Option<String>): TuiResult<()> {

// }
