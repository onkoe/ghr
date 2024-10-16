// Let's grab a system `Report` and output it as JSON!

use libghr::report::Report;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let report = Report::new().await.unwrap();
    let json_report = serde_json::to_string_pretty(&report).unwrap();
    println!("{}", json_report);
}
