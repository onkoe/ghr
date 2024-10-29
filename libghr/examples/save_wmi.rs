//! generates saves of wmi data to the hardcoded path.
//!
//! this can be used for unit testing!

use std::{collections::HashMap, io::Write, path::PathBuf};

use wmi::{COMLibrary, Variant};

fn main() {
    let com = COMLibrary::new().unwrap();
    let wmi = wmi::WMIConnection::new(com).unwrap();

    // put ur query here
    let query: Vec<HashMap<String, Variant>> =
        wmi.raw_query("SELECT * from Win32_Processor").unwrap();
    let stringd = serde_json::to_string(&query).unwrap();

    println!("{}", stringd);

    // save the query to a file
    let save_path = PathBuf::from("processors.json");
    let mut file = std::fs::File::create(save_path).unwrap();
    file.write_all(stringd.as_bytes()).unwrap();
}
