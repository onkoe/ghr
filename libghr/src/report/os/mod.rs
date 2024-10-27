//! `os`: Grabs operating system info from the computer.

use crate::prelude::internal::*;

use std::collections::BTreeMap;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_family = "unix")]
mod unix;

/// Information describing the operating system that's currently running.
///
/// On UNIX systems, this is derived from the `utsname` syscall.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct OperatingSystemInfo {
    name: String,
    distro_name: Option<String>,
    version: String,
    architecture: String,

    /// Additional stuff that's kinda operating-system dependent.
    other: BTreeMap<String, String>,
}

impl OperatingSystemInfo {
    /// Grabs the name of this operating system.
    ///
    /// ex: `Linux`
    #[tracing::instrument(skip(self))]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Grabs the version of this operating system.
    ///
    /// ex on Linux: `6.11.3-200.fc40.x86_64`
    #[tracing::instrument(skip(self))]
    pub fn version(&self) -> String {
        self.version.clone()
    }

    /// Grabs the architecture of this operating system.
    ///
    /// ex: `aarch64`
    #[tracing::instrument(skip(self))]
    pub fn arch(&self) -> String {
        self.architecture.clone()
    }

    #[tracing::instrument(skip(self))]
    /// This returns a list of stuff that comes from the operating system, but
    /// may be single-platform.
    ///
    /// You can might use this to list arbitrary capabilities and features of
    /// the OS.
    pub fn other(&self) -> BTreeMap<String, String> {
        self.other.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::report::Report;

    #[tokio::test]
    async fn runtime() {
        Report::os_info().await.unwrap();
    }
}
