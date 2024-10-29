use crate::prelude::internal::*;

use std::collections::BTreeMap;

impl Report {
    #[tracing::instrument]
    /// Grabs operating system info for the hardware report.
    #[cfg(target_os = "windows")]
    pub(crate) async fn os_info() -> GhrResult<OperatingSystemInfo> {
        let ver = tokio::task::spawn_blocking(windows_version::OsVersion::current)
            .await
            .unwrap();

        // use a simple env variable to grab this.
        // it's always around on Windows i guess!
        let arch = std::env::var("PROCESSOR_ARCHITECTURE")
            .map_err(|e| GhrError::OsInfoInaccessible(e.to_string()))?;

        Ok(OperatingSystemInfo {
            name: "Windows".into(),
            distro_name: None,
            version: format!("{}.{}.{}", ver.major, ver.minor, ver.build),
            architecture: arch,
            other: BTreeMap::new(),
        })
    }
}
