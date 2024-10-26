use libghr::report::Report;

#[tracing::instrument]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    // add logging
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // grab a report
    let report = Report::new().await.expect("report should work");

    // make it into json
    let json = serde_json::to_string(&report).expect("reports are representable as json");
    tracing::debug!("created json: {json}");
    // let params = [("report", json)];
    // let params = [json];

    // define the server to send it to
    const SERVER_IP: &str = "localhost";
    const SERVER_PORT: u16 = 8080;

    // make a reqwest client to speak w/ the server
    let client = reqwest::Client::new();

    // use reqwest to send our report
    let resp = client
        .post(format!("http://{SERVER_IP}:{SERVER_PORT}/add_report"))
        // .form(&params)
        .body(json)
        .send()
        .await;

    tracing::info!("response: {:#?}", resp);

    resp.expect("post should work")
        .error_for_status()
        .expect("status should be happy");
}
