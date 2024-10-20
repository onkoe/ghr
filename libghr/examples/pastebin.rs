use libghr::report::Report;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // add logging
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::WARN)
        .init();

    println!("welcome to the onkoe botnet, {}!", whoami::realname());

    // grab a report
    let report = Report::new()
        .await
        .expect("report blew up for some reason. report the errors above");

    // make it into json
    let json = serde_json::to_string(&report).expect("reports are representable as json");

    // make parameters
    let params = [
        (
            "title",
            format!("welcome to the onkoe botnet, {}", whoami::username()),
        ),
        ("content", json),
    ];

    // define the server to send it to
    const SERVER_IP: &str = "https://dpaste.com/api/v2/";

    // make a reqwest client to speak w/ the server
    let client = reqwest::Client::new();

    // use reqwest to send our report
    let resp = client.post(SERVER_IP).form(&params).send().await;

    let resp = resp
        .expect("post should work")
        .error_for_status()
        .expect("status should be happy");
    let headers = resp.headers();

    if let Some(Ok(url)) = headers.get("location").map(|url| url.to_str()) {
        println!("{url}")
    } else {
        println!("Failed to reach server. Please try again!");
    }

    println!("Press any key to continue...");
    let _ = rpassword::read_password();
}
