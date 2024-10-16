//! `os`: Grabs operating system info from the computer.

use crate::prelude::internal::*;

use std::collections::BTreeMap;

/// A `GhrResult` alias to ensure each platform implementation uses the same
/// return type.
pub type OsInfoReturnType = GhrResult<OperatingSystemInfo>;

impl Report {
    /// Grabs operating system info for the hardware report.
    #[cfg(target_os = "linux")]
    pub(crate) fn os_info() -> OsInfoReturnType {
        // we're on a Unix, so let's grab `utsname`

        use std::collections::BTreeMap;
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

        Ok(OperatingSystemInfo::new(name, release, arch, other_info))
    }

    /// Grabs operating system info for the hardware report.
    #[cfg(target_os = "windows")]
    pub(crate) fn os_info() -> OsInfoReturnType {
        let ver = windows_version::OsVersion::current();

        // use a simple env variable to grab this.
        // it's always around on Windows i guess!
        let arch = std::env::var("PROCESSOR_ARCHITECTURE")
            .map_err(|e| GhrError::OsInfoInaccessible(e.to_string()))?;

        Ok(OperatingSystemInfo {
            name: "Windows".into(),
            version: format!("{}.{}.{}", ver.major, ver.minor, ver.build),
            architecture: arch,
            other: BTreeMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::report::Report;

    #[test]
    fn compile() {
        Report::os_info().unwrap();
    }
}

/// Information describing the operating system that's currently running.
///
/// On UNIX systems, this is derived from the `utsname` syscall.
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct OperatingSystemInfo {
    name: String,
    version: String,
    architecture: String,

    /// Additional stuff that's kinda operating-system dependent.
    other: BTreeMap<String, String>,
}

impl OperatingSystemInfo {
    pub(crate) fn new(
        name: String,
        version: String,
        arch: String,
        other: BTreeMap<String, String>,
    ) -> Self {
        Self {
            name,
            version,
            architecture: arch,

            other,
        }
    }

    /// Grabs the name of this operating system.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Grabs the version of this operating system.
    pub fn version(&self) -> String {
        self.version.clone()
    }

    /// This returns a list of stuff that comes from the operating system, but
    /// may be single-platform.
    ///
    /// You can might use this to list arbitrary capabilities and features of
    /// the OS.
    pub fn other(&self) -> BTreeMap<String, String> {
        self.other.clone()
    }
}
