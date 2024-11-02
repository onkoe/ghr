use crate::{
    args::{Args, Subcommands},
    log::setup_logging,
    subcommands::{save, upload},
};
use libghr::Report;

use clap::Parser as _;

mod args;
mod log;
mod subcommands;

/// The default server where we'll upload reports.
const DEFAULT_SERVER: &str = "http://localhost:8080";

/// The default location where we'll save the report in JSON form.
const DEFAULT_FILE_NAME: &str = "ghr.json";

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[tokio::main]
async fn main() {
    // install panic handler
    human_panic::setup_panic!();

    // parse cli args
    let args = Args::parse();

    setup_logging().await;

    // run the command the user asked for
    match args.subcommands {
        Subcommands::Save { save_path } => save(save_path).await,
        Subcommands::Upload {
            server,
            confirm_without_prompt,
            save_path,
            shared,
        } => upload(server, confirm_without_prompt, save_path, shared).await,
    }
    .expect("a subcommand failed to execute");
}

/// Creates a report with the default settings.
async fn get_report() -> anyhow::Result<Report> {
    Ok(Report::new().await?)
}
