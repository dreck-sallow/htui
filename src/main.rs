use app_project::store::{LocalStore, Store};
use clap::Parser;
mod app_project;
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
        let local_store = LocalStore::new();
        let project_list = local_store.project_list().unwrap();

        for project in project_list {
            println!("{} - {}", project.id, project.name);
        }
    } else {
        if let Err(_err) = programs::tui_v2::run(cli.project).await {
            std::process::exit(1)
        }
        // if let Err(err) = tui::run_tui(cli.project).await {
        //     match err {
        //         tui::TuiError::Io(error) => eprintln!("IO_ERROR: {:?}", error),
        //         tui::TuiError::Config(error) => eprintln!("CONFIG_ERROR: {:?}", error),
        //     }
        //     std::process::exit(1)
        // }
    }
}
