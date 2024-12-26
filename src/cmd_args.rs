use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "htui")]
#[command(author, version, about, long_about = None)]
pub struct CmdArgs {
    #[arg(short, long)]
    pub url: Option<String>, // TODO: define url using library

    #[arg(value_enum, default_value = CmdHttpMethod::Get)]
    pub method: Option<CmdHttpMethod>,

    #[arg(short = 'H', long)]
    pub headers: Option<String>,

    #[arg(short = 'B', long)]
    pub body: Option<String>,

    #[arg(short = 'p', long)]
    pub project: Option<String>,
}

#[derive(Copy, Clone, ValueEnum)]
pub enum CmdHttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl CmdHttpMethod {
    pub fn as_str(&self) -> &'static str {
        //TODO: make upper case str, but error from ValueEnum
        match self {
            Self::Get => "get",
            Self::Post => "post",
            Self::Put => "put",
            Self::Patch => "patch",
            Self::Delete => "delete",
        }
    }
}

impl From<CmdHttpMethod> for clap::builder::OsStr {
    fn from(value: CmdHttpMethod) -> Self {
        clap::builder::OsStr::from(value.as_str())
    }
}
