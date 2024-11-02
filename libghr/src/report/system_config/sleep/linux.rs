use std::path::{Path, PathBuf};

use crate::prelude::internal::*;

/// gets info about the computer's sleep states.
pub(super) async fn get() -> Sleep {
    todo!()
}

/// gets info about the computer's sleep states.
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
fn parse_state_file(sleep: &mut Sleep, states: &String) {
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

struct SleepPaths {
    /// the path to `/sys/power/mem_sleep` (conf).
    mem_sleep: PathBuf,
    /// the path to `/sys/power/state` (conf).
    state: PathBuf,
}
