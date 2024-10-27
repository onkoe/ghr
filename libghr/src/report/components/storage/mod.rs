use crate::prelude::internal::*;

#[cfg(target_os = "linux")]
mod linux;

/// A description of a storage device.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct StorageDescription {
    /// The kind of storage device this is.
    pub kind: Option<StorageKind>,

    /// Information about the disk's usage and capacity.
    pub usage: StorageUsage,

    /// The rotation speed of the drive, if applicable, in RPM.
    pub speed: Option<u32>,

    /// The connector used by the drive.
    pub connector: Option<StorageConnector>,

    /// Whether or not the drive is known to be removable.
    pub is_removable: Option<bool>,
}

/// A "kind" describing a storage device.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub enum StorageKind {
    /// The drive reported itself to be solid-state, meaning it does not use
    /// moving parts.
    Ssd,

    /// The drive reported itself to be a hard drive using a "rotational"
    /// medium.
    Hdd,
}

/// A storage device's capacity + usage statistics.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct StorageUsage {
    /// The amount of storage capacity used, in KiB.
    pub usage: Option<u64>,

    /// The total storage capacity the device has available, in KiB.
    pub total_capacity: Option<u64>,
}

/// A storage device's connector.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub enum StorageConnector {
    Ide,
    Sata,
    /// "M.2" (NGFF): https://en.wikipedia.org/wiki/M.2
    M2,
    Pcie,
    Scsi,
    Other(String),
}

/// Grabs storage devices from the system.
#[tracing::instrument]
pub async fn get() -> GhrResult<Vec<ComponentInfo>> {
    #[cfg(target_os = "linux")]
    return linux::get().await;

    #[cfg(not(target_os = "linux"))]
    {
        tracing::error!("unimplemented.");
        Ok(Vec::new())
    }
}
