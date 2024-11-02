use std::path::{Path, PathBuf};

use acpi::fadt::Fadt;

use crate::prelude::internal::*;

/// gets info about the computer's sleep states.
#[tracing::instrument]
pub(super) async fn get() -> Sleep {
    todo!()
}

/// gets info about the computer's sleep states.
#[tracing::instrument]
async fn linux_sleep_info(paths: &SleepPaths) -> Sleep {
    let mut sleep = Sleep::default();

    // check for the supported types in `/sys/power/state`
    if let Some(states) = sysfs_value_opt::<String>(&paths.state).await {
        // we'll parse the `states` file and mutate our `Sleep`
        parse_state_file(&mut sleep, &states);
    }

    // if we have root, we can check for `s0i<x>` power states!
    //
    // let's make sure we have permissions.

    todo!()
}

/// mutates the given `Sleep` to account for the info in `/sys/power/states`.
#[tracing::instrument]
fn parse_state_file(sleep: &mut Sleep, states: &str) {
    // we need to check for various strings
    //
    // s0: software suspend
    sleep.s0 = if states.contains("freeze") | states.contains("s2idle") {
        SleepMode::Supported
    } else {
        SleepMode::Unsupported
    };

    // s1: naive standby
    sleep.s1 = if states.contains("shallow") | states.contains("standby") {
        SleepMode::Supported
    } else {
        SleepMode::Unsupported
    };

    // s2: naive standby with cpu powered down.
    //
    // linux doesn't seem to report this:
    // https://www.kernel.org/doc/Documentation/power/states.txt
    sleep.s2 = SleepMode::Unknown;

    // s3: suspend-to-ram
    sleep.s3 = if states.contains("deep") {
        SleepMode::Supported
    } else {
        SleepMode::Unsupported
    };

    // s4: suspend-to-disk (hibernation!)
    sleep.s4 = if states.contains("disk") {
        SleepMode::Supported
    } else {
        SleepMode::Unsupported
    };
}

/// attempts to read acpi data from disk. this requires root or weird fs perms.
///
/// `path` should be the `/sys/firmware/acpi/tables` directory.
#[tracing::instrument]
async fn read_acpi(path: &Path) -> Option<Fadt> {
    // first of all, make sure we can even read the tables
    let fadp = match tokio::fs::File::open(path.join("FACP")).await {
        Ok(f) => f,
        Err(e) => {
            // we'll print a more helpful log if permissions aren't right.
            if let std::io::ErrorKind::PermissionDenied = e.kind() {
                tracing::info!("We don't have permission to read the ACPI table. (err: {e})");
            } else {
                tracing::info!(
                    "Failed to access ACPI tables. The system may not support it. (err: {e})"
                );
            }
            // if we can't read the files, there's nothing to be done here.
            return None;
        }
    };

    // we can read them, then! let's parse our `fadp` into a table
    let table = todo!();
    // oh. just realized `fadp` doesn't contain info about s0ix.
    // so it's not useful for me D:
}

/// parses acpi data.
async fn parse_acpi(sleep: &mut Sleep, info: Fadt) {
    todo!()
}

#[derive(Debug)]
struct SleepPaths {
    /// the path to `/sys/power/mem_sleep` (conf).
    mem_sleep: PathBuf,
    /// the path to `/sys/power/state` (conf).
    state: PathBuf,
    /// the path to `/sys/firmware/acpi/tables`
    tables: PathBuf,
}
