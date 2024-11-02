use std::io;

use clap::Parser;
use cli_app::CliAppOptions;
use serde::Serialize;

mod cli_app;
mod error;
mod store;
mod tui;

#[derive(Parser, Debug)]
#[command(name = "HTUI")]
#[command(author, version, about)]
struct Cli {
    /// Url to make the request
    #[arg(short, long)]
    url: Option<String>,

    // headers: Option<HashMap<String, String>>,
    /// Http method to execute
    method: Option<CliHttpMethod>,

    /// Headers used to make the request
    #[arg(short = 'H', long)]
    headers: Option<String>,

    /// Body data used to make the request
    #[arg(short = 'd', long, value_delimiter = ',')]
    body: Option<String>,
}
#[derive(clap::ValueEnum, Debug, Clone, Default, Serialize)]
#[serde(rename_all = "UPPERCASE")]
enum CliHttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
}

impl From<CliHttpMethod> for reqwest::Method {
    fn from(value: CliHttpMethod) -> Self {
        match value {
            CliHttpMethod::Get => reqwest::Method::GET,
            CliHttpMethod::Post => reqwest::Method::POST,
            CliHttpMethod::Put => reqwest::Method::PUT,
            CliHttpMethod::Delete => reqwest::Method::DELETE,
            CliHttpMethod::Patch => reqwest::Method::PATCH,
            CliHttpMethod::Head => reqwest::Method::HEAD,
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();
    if cli.url.is_some() {
        let options = CliAppOptions::try_from(cli).expect("error for options");
        cli_app::run_app(options).await
    } else {
        run_tui_app().await
    }
}

async fn run_tui_app() -> io::Result<()> {
    tui::run_app().await?;
    println!("Launch the TUI application");
    Ok(())
}
