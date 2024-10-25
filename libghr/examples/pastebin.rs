use std::path::PathBuf;

use clap::Parser;
use libghr::report::Report;

/// Simple program to greet a person
#[derive(Clone, Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Pretty-prints the JSON to the command line.
    #[arg(short, long, default_value_t = false)]
    pretty_print: bool,

    /// Saves a readable version to disk at this location.
    #[arg(short, long)]
    save_to: Option<String>,

    /// Whether or not to upload the file to the web.
    #[arg(short, long, default_value_t = true)]
    upload: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // add logging
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::WARN)
        .init();

    // check args
    let args = Args::parse();

    println!("welcome to the onkoe botnet, {}!", whoami::realname());

    // grab a report
    let report = Report::new()
        .await
        .expect("report blew up for some reason. report the errors above");

    // make it into json
    let json: String = serde_json::to_string(&report).expect("reports are representable as json");

    // if we have the arg set, we'll pretty print it
    let pretty_json = serde_json::to_string_pretty(&report).expect("reports are rep as json");

    if args.pretty_print {
        print!("{}", pretty_json);
    }

    // and save it to disk, if asked
    if let Some(path) = args.save_to {
        // make a path from the string
        let path = PathBuf::from(path);

        // move the location to a file if needed
        let path = if path.is_dir() {
            path.join("ghr.json")
        } else {
            path
        };

        tokio::fs::write(&path, pretty_json)
            .await
            .expect("user given path should exist");

        println!("Saved report file to {}", path.display());
    }

    // only upload if the user says to
    if args.upload {
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
    }

    println!("All done! Press any key to continue...");
    let _ = rpassword::read_password();
}
