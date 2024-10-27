use crate::prelude::internal::*;

use std::collections::BTreeMap;
use std::path::Path;

impl Report {
    /// Grabs operating system info for the hardware report.
    #[tracing::instrument]
    pub(crate) async fn os_info() -> GhrResult<OperatingSystemInfo> {
        // we're on a Unix, so let's grab `utsname`
        let uname = match nix::sys::utsname::uname() {
            Ok(uts) => uts,
            Err(e) => return Err(GhrError::OsInfoInaccessible(e.to_string())),
        };

        // form the type from the elements we got
        let name = uname.sysname().to_string_lossy().to_string();
        let release = uname.release().to_string_lossy().to_string();
        let arch = uname.machine().to_string_lossy().to_string();
        let other_info = BTreeMap::from([
            (
                "version".into(),
                uname.version().to_string_lossy().to_string(),
            ),
            // TODO: make sure we want these on the web
            (
                "domainname".into(),
                uname.domainname().to_string_lossy().to_string(),
            ),
            (
                "nodename".into(),
                uname.nodename().to_string_lossy().to_string(),
            ),
        ]);

        // we'll also grab the "real" name of the system
        let distro_name = OperatingSystemInfo::find_distro_name("/etc/os-release").await;

        Ok(OperatingSystemInfo {
            name,
            distro_name,
            version: release,
            architecture: arch,
            other: other_info,
        })
    }
}

impl OperatingSystemInfo {
    /// finds the `distro_name` field, like "Fedora Linux 40 (Forty)".
    #[tracing::instrument]
    async fn find_distro_name<P: AsRef<Path> + std::fmt::Debug>(
        os_release_path: P,
    ) -> Option<String> {
        // read `os-release` from the given path
        let os_release = Self::read_os_release(os_release_path).await?;

        // find the `NAME="Distro Name"` part of the file
        let line = Self::os_release_field(os_release, "PRETTY_NAME").await?;

        // clean it up
        let Some(quoted) = line.split("=").last() else {
            tracing::warn!("Line didn't have an equals sign! (line: `{line}`).");
            return None;
        };
        let whitespaced = quoted.replace('\"', "");

        // send it off
        Some(whitespaced.trim().to_string())
    }

    /// reads the `os-release` file given and returns it, in entirety, as a
    /// string if found.
    #[tracing::instrument]
    async fn read_os_release<P: AsRef<Path> + std::fmt::Debug>(
        os_release_path: P,
    ) -> Option<String> {
        let os_release_path = os_release_path.as_ref();

        let Some(os_release) = sysfs_value_opt::<String>(os_release_path).await else {
            tracing::warn!(
                "Failed to navigate to `os-release` file at {}",
                os_release_path.display()
            );
            return None;
        };

        Some(os_release)
    }

    /// finds some key in the given `os-release` file.
    ///
    /// if found, it'll return it as a string.
    #[tracing::instrument]
    async fn os_release_field(os_release: String, key: &str) -> Option<String> {
        let Some(field) = os_release.lines().find(|l| l.contains(&format!("{key}="))) else {
            tracing::warn!("Failed to find line with key `{key}` in `os-release` file.");
            return None;
        };

        Some(field.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[tokio::test]
    async fn get_distro_name() {
        let crate_root = env!("CARGO_MANIFEST_DIR");
        let path = PathBuf::from(crate_root).join("tests/assets/linux/sysfs/etc/os-release");
        let distro_name = OperatingSystemInfo::find_distro_name(path).await.unwrap();

        assert_eq!(distro_name, "GHR Linux 40 (Forty)");
    }
}
