use std::path::PathBuf;

use crate::prelude::internal::*;

/// gets info about the computer's sleep states.
#[tracing::instrument]
pub(super) async fn get() -> Sleep {
    let paths = SleepPaths {
        state: PathBuf::from("/sys/power/state"),
    };

    linux_sleep_info(&paths).await
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

    sleep
}

/// mutates the given `Sleep` to account for the info in `/sys/power/state`.
#[tracing::instrument]
fn parse_state_file(sleep: &mut Sleep, states: &str) {
    // we need to check for various strings.
    //
    // s0: software suspend
    sleep.s0 = (states.contains("freeze") || states.contains("s2idle")).into();

    // s1: naive standby
    sleep.s1 = (states.contains("shallow") || states.contains("standby")).into();

    // s2: naive standby with cpu powered down.
    //
    // linux doesn't seem to report this:
    // https://www.kernel.org/doc/Documentation/power/states.txt
    sleep.s2 = SleepMode::Unknown;

    // s3: suspend-to-ram
    sleep.s3 = states.contains("deep").into();

    // s4: suspend-to-disk (hibernation!)
    sleep.s4 = states.contains("disk").into();
}

#[derive(Debug)]
#[non_exhaustive]
struct SleepPaths {
    /// the path to `/sys/power/state`.
    state: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn check_states() {
        let paths = SleepPaths {
            state: sysfs_power_path().join("state"),
        };

        // grab our possible sleep states
        let sleep = linux_sleep_info(&paths).await;

        // https://www.kernel.org/doc/Documentation/power/states.txt
        //
        // - freeze: s0/software suspend
        // - mem: currently unused.
        // - disk: s4/hibernation

        let expected = Sleep {
            s0: SleepMode::Supported,
            s1: SleepMode::Unsupported,
            s3: SleepMode::Unsupported,
            s4: SleepMode::Supported,
            ..Default::default()
        };

        assert_eq!(expected, sleep);
    }

    #[tracing::instrument]
    fn sysfs_power_path() -> PathBuf {
        let root = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(format!("{root}/tests/assets/linux/sysfs/sys/power"))
    }
}
