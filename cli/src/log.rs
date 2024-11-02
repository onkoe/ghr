use std::{fs::OpenOptions, path::PathBuf};

use tracing_subscriber::{
    fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter, Layer as _,
};

use crate::built_info;

/// Starts logging.
pub async fn setup_logging() {
    // find a nice place to drop our temp files (like logs)
    let logs_directory = log_location();

    // create it if it doesn't exist yet
    _ = tokio::fs::create_dir_all(&logs_directory).await;

    // in debug mode, we'll say where we save the logs
    #[cfg(debug_assertions)]
    println!(
        "msg because of debug mode: Logs are being saved to `{}`.",
        logs_directory.display()
    );

    // set up log files.
    //
    // `ghr.latest.log` is the current log file. we'll overwrite it if it's
    // there already.
    let log_file_latest = OpenOptions::new()
        .append(true)
        .create(true)
        .open(logs_directory.join("ghr.latest.log"))
        .unwrap();
    let log_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(logs_directory.join(format!(
            "ghr.{}.log",
            chrono::Local::now().format("%Y-%m-%d__%H-%M-%S")
        )))
        .unwrap();
    let (latestfile, _s) = tracing_appender::non_blocking(log_file_latest);
    let (datefile, _s) = tracing_appender::non_blocking(log_file);

    // enable logging.
    //
    // note that we want to log everything to a file, and only OUR (`cli`)
    // errors to the terminal. otherwise, the output will be kinda awful...
    tracing_subscriber::Registry::default()
        .with(
            // log to file (datetime visible)
            fmt::layer().pretty().with_writer(datefile),
        )
        .with(
            // log to file ("latest")
            fmt::layer().pretty().with_writer(latestfile),
        )
        .with(
            // log ERRORS from THIS CRATE to the terminal
            tracing_subscriber::fmt::layer().pretty().with_filter(
                EnvFilter::builder()
                    .parse(constcat::concat!(built_info::PKG_NAME, "=ERROR"))
                    .unwrap(),
            ),
        )
        .init();
}

/// Finds the location where we'll store logs.
///
/// This is generally in `$HOME/.cache/ghr/`
pub fn log_location() -> PathBuf {
    dirs::cache_dir()
        .expect("the system should have a cache directory")
        .join(built_info::PKG_NAME)
}

/// Indicates the location of the 'latest' log file.
pub fn latest_log_location() -> String {
    log_location().join("ghr.latest.log").display().to_string()
}
