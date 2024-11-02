use std::path::Path;

use crate::log::latest_log_location;
use libghr::Report;

use colored::Colorize as _;
use tokio::io::AsyncWriteExt as _;

/// Saves the `Report` to disk as JSON.
pub async fn run(save_path: &Path) -> anyhow::Result<()> {
    let report = Report::new().await?;

    // we already print a success message if we write to disk.
    //
    // let's also do so if we fail
    if let Err(e) = write_report_to_disk(save_path, &report).await {
        tracing::warn!("Failed to write report to disk at `{save_path:?}`. (err: {e})");
        println!(
            "{}{}{}",
            "Failed to write report to disk. Please see the error log at `".red(),
            latest_log_location().white(),
            "` for more information.".red()
        );
    }

    Ok(())
}

/// Attempts to write the report to disk.
///
/// Prints the path it was written to.
pub async fn write_report_to_disk(save_path: &Path, report: &Report) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(report).inspect_err(|e| {
        tracing::warn!("Failed to make report into a pretty JSON string. (err: {e})")
    })?;

    if let Ok(meta) = tokio::fs::metadata(save_path).await {
        if meta.is_dir() {
            tracing::warn!("User passed in a directory. Warning them...");
            println!(
                "{}{}{}",
                "The save path you passed in is an existing folder. Please provide a ".yellow(),
                "file name".bright_yellow(),
                " instead.".yellow()
            );
            return Ok(());
        }
    }

    // create the path if it doesn't exist
    let mut f = tokio::fs::File::create(save_path)
        .await
        .inspect_err(|e| tracing::warn!("Failed to create file for report on disk! (err: {e})"))?;

    // write it to disk
    f.write_all(json.as_bytes())
        .await
        .inspect_err(|e| tracing::warn!("Failed to write report to disk! (err: {e})"))?;

    println!(
        "{}{}{}",
        "This computer's hardware report has been written to disk at `".green(),
        save_path.to_string_lossy().bright_blue(),
        "`.".green()
    );

    Ok(())
}
