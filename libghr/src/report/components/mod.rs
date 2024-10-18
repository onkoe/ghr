use fraction::Decimal;
use sysinfo::{CpuRefreshKind, RefreshKind};

use crate::prelude::internal::*;

pub mod cpu;
pub mod pci;
pub mod usb;

/// Grabs any known components (devices) on the system.
///
/// Currently, this just supports USB and PCI. Additional device types will
/// come soon!
pub async fn get_components() -> GhrResult<Vec<ComponentInfo>> {
    let system = sysinfo::System::new_with_specifics(
        RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
    );

    let (cpu, usb, pci) = tokio::try_join! {
        cpu::cpu(&system),
        usb::usb_components(),
        pci::pci_components(),
    }?;

    Ok([vec![cpu], usb, pci].into_iter().flatten().collect())
}

/// A representation of any single component in the machine.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct ComponentInfo {
    /// The type of bus this component is from.
    bus: ComponentBus,

    /// An identifier for the device.
    id: String,

    /// Info about what kind of device this is. (TODO: make a type for this and parse on linux/android)
    class: String,

    /// The device's vendor identifier.
    vendor_id: String,

    /// Status info about the component.
    status: ComponentStatus,

    /// General information about the component.
    desc: ComponentDescription,
}

/// The bus a component is on.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum ComponentBus {
    Pci,
    Usb,
    Sys,
    Ps2,
    Serial,
    Eisa,

    // mostly hard drive stuff
    Nvme,
    Scsi,
    Ide,

    Other(String),
}

/// Information about the health of the component.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct ComponentStatus {}

/// A general 'description' about the component
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum ComponentDescription {
    CpuDescription {
        /// The CPU's clock speed in GHz. (ex: 3.4 GHz)
        clock_speed: f64,

        /// The CPU's core count.
        core_ct: u32,
    },

    /// No description is available for this device.
    None,
}

// all this helps with accessing devices on linux
#[cfg(target_os = "linux")]
mod linux {
    use crate::prelude::internal::*;

    use std::path::Path;
    use tokio::fs::DirEntry;
    use tokio_stream::{wrappers::ReadDirStream, StreamExt};

    pub(super) async fn devices(path: impl AsRef<Path>) -> GhrResult<Vec<DirEntry>> {
        let all_devices = tokio::fs::read_dir(path)
            .await
            .map_err(|e| GhrError::ComponentInfoInaccessible(e.to_string()))?;

        Ok(ReadDirStream::new(all_devices)
            .filter_map(|dev| dev.ok())
            .collect::<Vec<_>>()
            .await)
    }
}
