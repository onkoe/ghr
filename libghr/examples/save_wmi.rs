//! generates saves of wmi data to the hardcoded path.
//!
//! this can be used for unit testing!

#[cfg(target_os = "windows")]
#[tracing::instrument]
fn main() {
    use std::{collections::HashMap, io::Write, path::PathBuf};
    use wmi::{COMLibrary, Variant};

    let com = COMLibrary::new().unwrap();
    let wmi = wmi::WMIConnection::new(com).unwrap();

    // put ur query here
    let query: Vec<HashMap<String, Variant>> =
        wmi.raw_query("SELECT * from Win32_DiskDrive").unwrap();
    let stringd = serde_json::to_string(&query).unwrap();

    println!("{}", stringd);

    // save the query to a file
    let save_path = PathBuf::from("libghr/tests/assets/windows/sls2_nvme_storage.json");
    let mut file = std::fs::File::create(save_path).unwrap();
    file.write_all(stringd.as_bytes()).unwrap();
}

#[cfg(not(target_os = "windows"))]
#[tracing::instrument]
fn main() {
    println!("why are you running this");
}
