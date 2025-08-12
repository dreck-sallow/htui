use clap::Parser;
use paths::Paths;
use programs::tui;
use store::{LocalStore, Store};

mod paths;
mod programs;
mod store;

#[derive(Parser)]
#[command(name = "POSTUI")]
#[command(version, about)]
struct CliOptions {
    pub project: Option<String>,

    #[arg(short, long)]
    pub list_projects: bool,
}

#[tokio::main]
async fn main() {
    let cli = CliOptions::parse();

    if cli.list_projects {
        let local_store = LocalStore::new(Paths::new("store"));
        let project_list = local_store.project_list().await.unwrap();

        for project in project_list {
            println!("{} - {}", project.id, project.name);
        }
    } else {
        if let Err(err) = tui::run_tui(cli.project).await {
            match err {
                tui::TuiError::Io(error) => eprintln!("IO_ERROR: {:?}", error),
                tui::TuiError::Config(error) => eprintln!("CONFIG_ERROR: {:?}", error),
            }
            std::process::exit(1)
        }
    }
}
