use crate::prelude::internal::*;

pub mod cpu;
pub mod gpu;
pub mod pci;
pub mod ram;
pub mod usb;

/// Grabs any known components (devices) on the system.
///
/// Currently, this just supports USB and PCI. Additional device types will
/// come soon!
pub async fn get_components() -> GhrResult<Vec<ComponentInfo>> {
    let (cpu, usb, pci, ram) = tokio::try_join! {
        cpu::cpu(),
        usb::usb_components(),
        pci::pci_components(),
        ram::ram(),
    }?;

    Ok([cpu, usb, pci, ram].into_iter().flatten().collect())
}

/// A representation of any single component in the machine.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct ComponentInfo {
    /// The type of bus this component is from.
    bus: ComponentBus,

    /// An identifier for the device.
    id: Option<String>,

    /// Info about what kind of device this is.
    class: Option<String>,

    /// The device's vendor identifier.
    vendor_id: Option<String>,

    /// Status info about the component.
    status: Option<ComponentStatus>,

    /// General information about the component.
    desc: ComponentDescription,
}

/// The bus a component is on.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
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
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct ComponentStatus {}

/// A general 'description' about the component
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub enum ComponentDescription {
    /// About the central processing unit (CPU).
    CpuDescription(CpuDescription),

    /// For the system memory.
    ///
    /// Some info is unavailable since some machines lack the ability to share
    /// per-stick info.
    ///
    /// Also, all values are in bytes.
    RamDescription(RamDescription),

    /// About the graphics card (GPU).
    GpuDescription(GpuDescription),

    /// No description is available for this device.
    None,
}

/// Whether or not a component is removable.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
pub enum Removability {
    /// You can remove this component from your computer without damaging the
    /// hardware.
    Removable,
    /// This component is not known to be removable.
    NonRemovable,
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

#[cfg(target_os = "windows")]
pub(crate) mod windows {
    use std::collections::HashMap;

    use crate::{
        error::{GhrError, GhrResult},
        report::{ComponentBus, ComponentDescription},
    };
    use wmi::{COMLibrary, Variant, WMIConnection};

    use super::ComponentInfo;

    // holds the thread-local reference to the com library.
    //
    // see the `COMLibrary` docs for additional information.
    thread_local! {
        pub(super) static COM_LIBRARY: GhrResult<COMLibrary> = {
        match COMLibrary::new() {
            Ok(com) => Ok(com),
            Err(e) => {
                tracing::error!("Failed to open connection to the Windows COM library.");
                Err(GhrError::ComponentInfoInaccessible(e.to_string()))
            }
        }};
    }

    pub(crate) fn get_wmi() -> GhrResult<WMIConnection> {
        // connect to the windows stuff
        let com_library = COM_LIBRARY.with(|com| com.clone())?;
        let wmi_connection = match WMIConnection::new(com_library) {
            Ok(wmi) => wmi,
            Err(e) => {
                tracing::error!("Couldn't connect to the WMI service.");
                return Err(GhrError::ComponentInfoInaccessible(e.to_string()));
            }
        };

        Ok(wmi_connection)
    }

    /// grabs all "plug and play" devices on a windows computer
    pub(super) async fn get_pnp(wmi: WMIConnection) -> GhrResult<Vec<HashMap<String, Variant>>> {
        let query: Result<Vec<HashMap<String, Variant>>, _> =
            wmi.async_raw_query("SELECT * FROM Win32_PnPEntity").await;

        match query {
            Ok(devices) => Ok(devices),
            Err(e) => {
                tracing::error!("Failed to get Plug and Play devices from Windows.");
                Err(GhrError::ComponentInfoInaccessible(e.to_string()))
            }
        }
    }

    /// checks for "pnp" devices with a prefix
    pub(super) async fn get_pnp_with_did_prefix(
        wmi: WMIConnection,
        prefix: &str,
    ) -> GhrResult<Vec<ComponentInfo>> {
        let query = get_pnp(wmi).await;
        pnp_filter_did_prefix(query?, prefix).await
    }

    async fn pnp_filter_did_prefix(
        query: Vec<HashMap<String, Variant>>,
        did_prefix: &str,
    ) -> GhrResult<Vec<ComponentInfo>> {
        // filter devices for the pci ones
        tracing::debug!("filtering pci devices...");

        Ok(query
            .into_iter()
            .map(|pnp_device| (pnp_device.get("DeviceID").string_from_variant(), pnp_device))
            .filter(|(device_id, _pnp_device)| {
                device_id
                    .clone()
                    .filter(|did| did.trim().to_uppercase().starts_with(did_prefix))
                    .is_some()
            })
            .map(|(_, pnp_device)| pnp_device)
            .map(|pci_device| {
                // grab important details
                let id = pci_device.get("Name").string_from_variant();
                let class = pci_device.get("PNPClass").string_from_variant();
                let vendor_id = pci_device.get("Manufacturer").string_from_variant();

                let bus = match did_prefix {
                    "USB" => ComponentBus::Usb,
                    "PCI" => ComponentBus::Pci,
                    _ => ComponentBus::Sys,
                };

                ComponentInfo {
                    bus,
                    id,
                    class,
                    vendor_id,
                    status: None,
                    desc: ComponentDescription::None,
                }
            })
            .collect::<Vec<ComponentInfo>>())
    }

    /// A simple trait that makes it easier to get/map values from a
    /// `wmi::Variant`.
    pub(crate) trait VariantInto {
        /// tries to get a string from this variant
        fn string_from_variant(&self) -> Option<String>;

        /// tries to get a u32 from this variant
        fn u32_from_variant(&self) -> Option<u32>;

        /// tries to get a u64 from this variant
        fn u64_from_variant(&self) -> Option<u64>;

        /// tries to get a bool from this variant
        fn bool_from_variant(&self) -> Option<bool>;
    }

    impl VariantInto for Variant {
        fn string_from_variant(&self) -> Option<String> {
            if let Variant::String(s) = self {
                Some(s.clone())
            } else {
                None
            }
        }

        fn u32_from_variant(&self) -> Option<u32> {
            if let Variant::UI4(u) = *self {
                Some(u)
            } else {
                None
            }
        }

        fn u64_from_variant(&self) -> Option<u64> {
            if let Variant::UI8(u) = *self {
                Some(u)
            } else {
                None
            }
        }

        fn bool_from_variant(&self) -> Option<bool> {
            if let Variant::Bool(b) = *self {
                Some(b)
            } else {
                None
            }
        }
    }

    impl VariantInto for Option<&Variant> {
        fn string_from_variant(&self) -> Option<String> {
            if let Some(Variant::String(s)) = self {
                return Some(s.clone());
            }

            None
        }

        fn u32_from_variant(&self) -> Option<u32> {
            if let Some(Variant::UI4(u)) = self {
                return Some(*u);
            }

            None
        }

        fn u64_from_variant(&self) -> Option<u64> {
            if let Some(Variant::UI8(u)) = self {
                return Some(*u);
            }

            None
        }

        fn bool_from_variant(&self) -> Option<bool> {
            if let Some(Variant::Bool(b)) = self {
                return Some(*b);
            }

            None
        }
    }
}
