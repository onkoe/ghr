// Let's grab a system `Report` and output it as JSON!

use libghr::report::Report;
use tracing::Level;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // turn on logging
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(Level::DEBUG)
        .init();

    // make the report
    let report = Report::new().await.unwrap();
    let json_report = serde_json::to_string_pretty(&report).unwrap();
    println!("{}", json_report);
}
