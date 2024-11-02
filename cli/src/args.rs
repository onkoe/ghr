use std::path::PathBuf;

use clap::builder::styling::AnsiColor;

use crate::{DEFAULT_FILE_NAME, DEFAULT_SERVER};

#[derive(Clone, Debug, clap::Parser)]
#[command(styles = clap_style())]
pub struct Args {
    #[clap(subcommand)]
    pub subcommands: Subcommands,
}

/// Command-line arguments for the CLI.
#[derive(Clone, Debug, clap::Subcommand)]
pub enum Subcommands {
    /// Creates and saves a system hardware report to disk.
    Save {
        /// A path where we'll save the report in its JSON form.
        #[arg(long, default_value = DEFAULT_FILE_NAME)]
        save_path: PathBuf,
    },
    /// Uploads a hardware report to the web.
    Upload {
        /// The server where we'll upload reports.
        #[arg(short, long, default_value = DEFAULT_SERVER)]
        server: String,

        /// Whether we'll upload without confirmation.
        ///
        /// Example: If you pass `ghr upload -y`, the tool will upload your
        /// system information to the web without making you review it first.
        #[arg(short = 'y', long = "confirm", default_value_t = false)]
        confirm_without_prompt: bool,

        /// A path where we'll save the report in its JSON form.
        ///
        /// If not passed, we will not save a file to disk, and will only
        /// attempt to upload to the web.
        #[arg(long)]
        save_path: Option<PathBuf>,

        #[command(flatten)]
        shared: SharedArgs,
    },
}

/// Args shared between the `Save` and `Upload` subcommands.
///
/// These are flattened, meaning this struct doesn't actually appear for users.
#[derive(Clone, Debug, clap::Args)]
pub struct SharedArgs {
    // TODO: add options:
    //    - use mac address
}

/// makes a "style" for clap. that's just the colors we want to use
const fn clap_style() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .header(AnsiColor::Green.on_default())
        .usage(AnsiColor::BrightGreen.on_default())
        .literal(AnsiColor::Blue.on_default())
        .placeholder(AnsiColor::BrightBlue.on_default())
}
