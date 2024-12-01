use clap::Parser;

//modules
mod cmd_args;
mod tools;

use tools::{cli, tui};

#[tokio::main]
async fn main() {
    let cmd_cli_args = cmd_args::CmdArgs::parse();

    if let Some(_url) = cmd_cli_args.url.clone() {
        let cli_params = cmd_cli_args.try_into().unwrap();
        cli::run(cli_params).await;
    } else {
        tui::run().await.unwrap();
    }

    println!("main function");
}
