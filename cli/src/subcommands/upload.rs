use std::path::PathBuf;

use colored::Colorize as _;

use crate::{args::SharedArgs, get_report, log::log_location};

pub async fn upload(
    server: String,
    confirm_without_prompt: bool,
    _save_path: Option<PathBuf>,
    _shared: SharedArgs,
) -> anyhow::Result<()> {
    // grab a report
    let report = get_report().await?;

    // make sure the user actually wants to upload this
    if !confirm_without_prompt {
        print!(
            "{}{}{}",
            "Are you sure you wish to upload this hardware report to the online database at `"
                .blue(),
            server.as_str().bright_blue(),
            "`? ".blue(),
        );
        // grab input until it's good
        loop {
            println!("{}", "[Y/n]".blue()); // print the prompt

            let mut input = String::new();
            let result = std::io::stdin().read_line(&mut input).inspect_err(|e| {
                tracing::warn!(
                    "User gave weird input to `read_line`, 
                    so couldn't parse to string. (err: {e})"
                )
            });

            if let Ok(_) = result {
                match input.trim().to_ascii_lowercase().as_str() {
                    "yes" | "y" | "t" | "true" | "yuh" | "" => {
                        break;
                    }

                    "no" | "n" | "f" | "false" => {
                        println!("{}", "Cancelled upload.".green());
                        return Ok(());
                    }

                    // ask again if the response was weird
                    _ => {}
                };
            }

            println!(); // add blank line each loop
        }
    }

    // ok, we're all good to upload the file!
    //
    // first, convert to json
    let report_as_json = serde_json::to_string(&report)
        .inspect_err(|e| tracing::error!("Failed to convert `Report` into JSON. (err: {e})"))?;

    // and now send it
    let resp = match reqwest::Client::new()
        .post(format!("{server}/add_report"))
        .body(report_as_json)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map(|r| async { r.text().await })
        .inspect_err(|e| tracing::warn!("Failed to upload the report. (err: {e})"))
    {
        Ok(r) => r.await,
        Err(e) => {
            tracing::warn!("Couldn't upload the report. (err: {e})");
            println!(
                "{}",
                "Failed to upload the report. Please check your internet connection.".red()
            );
            println!(
                "See the log at `{}` for more info.",
                log_location().display().to_string().bright_black()
            );
            return Ok(());
        }
    };

    match resp {
        Ok(r) => {
            println!(
                "{}{r}",
                "Your report has been added to the database! See it here: ".green(),
            );
        }
        Err(e) => {
            tracing::warn!("Couldn't read server response. (err: {e})");

            println!(
                "{}",
                "Couldn't read the response for the database entry.".red()
            );
            println!(
                "See the log at `{}` for more info.",
                log_location().display().to_string().bright_black()
            );
        }
    }

    Ok(())
}
