// Let's grab a system `Report` and output it as JSON!

use std::path::PathBuf;

use libghr::report::Report;
use tokio::io::AsyncWriteExt;
use tracing::Level;

#[tracing::instrument]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    // turn on logging
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(Level::DEBUG)
        .init();

    // make the report
    let report = Report::new().await.unwrap();

    // create a dir for the output
    let path = PathBuf::from(format!(
        "./ghr-{}",
        time::OffsetDateTime::now_utc()
            .to_string()
            .replace(":", ";")
    ));
    _ = tokio::fs::create_dir_all(&path).await;

    // push pretty print
    {
        let pretty = serde_json::to_string_pretty(&report).unwrap();
        let mut pretty_file = tokio::fs::File::create(&path.join("ghr.pretty.json"))
            .await
            .unwrap();
        pretty_file.write_all(pretty.as_bytes()).await.unwrap();
    }

    // and the ugly one
    {
        let json = serde_json::to_string_pretty(&report).unwrap();
        let mut ugly_file = tokio::fs::File::create(&path.join("ghr.json"))
            .await
            .unwrap();
        ugly_file.write_all(json.as_bytes()).await.unwrap();
    }

    // compress the folder
    let comp_filename = path.file_name().unwrap().to_string_lossy();
    let compressed_path = {
        let mut p = path.clone();
        p.pop();
        p.join(format!("{}.7z", comp_filename))
    };
    sevenz_rust::compress_to_path(&path, &compressed_path).expect("7zip failed");

    println!("Saving to {}...", compressed_path.display());

    println!("All done! Press any key to continue...");
    _ = rpassword::read_password();
}
