use futures::{stream::FuturesUnordered, FutureExt as _, StreamExt};

use crate::prelude::internal::*;

pub mod cpu;
pub mod gpu;
pub mod nic;
pub mod pci;
pub mod psu;
pub mod ram;
pub mod storage;
pub mod usb;

#[tracing::instrument]
/// Grabs any known components (devices) on the system.
pub async fn get_components() -> GhrResult<Vec<ComponentInfo>> {
    let mut futures = FuturesUnordered::new();

    // add components to the set. this prevents a stack overflow on a shared
    // await point!
    futures.push(cpu::get().boxed_local());
    futures.push(usb::get().boxed_local());
    futures.push(pci::get().boxed_local());
    futures.push(ram::get().boxed_local());
    futures.push(gpu::get().boxed_local());
    futures.push(psu::get().boxed_local());
    futures.push(storage::get().boxed_local());
    futures.push(nic::get().boxed_local());

    // iterate over each future in the stream.
    let mut components = Vec::new();
    while let Some(comp) = futures.next().await {
        components.push(comp?);
    }

    Ok(components
        .into_iter()
        .flatten()
        .filter(|c: &ComponentInfo| !c.is_blank())
        .collect())
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

impl ComponentInfo {
    #[tracing::instrument(skip(self))]
    /// Checks if a component is "blank" - meaning it has no fields filled out.
    ///
    /// In other words, if all of its fields are `None`.
    pub fn is_blank(&self) -> bool {
        self.id.is_none()
            && self.class.is_none()
            && self.vendor_id.is_none()
            && self.status.is_none()
            && matches!(self.desc, ComponentDescription::None)
    }

    /// Gets the "bus" this component this was attached to during report creation.
    #[tracing::instrument(skip(self))]
    pub fn bus(&self) -> ComponentBus {
        self.bus.clone()
    }

    /// Gets the name for this component.
    #[tracing::instrument(skip(self))]
    pub fn id(&self) -> Option<String> {
        self.id.clone()
    }

    /// Gets the class for this component.
    #[tracing::instrument(skip(self))]
    pub fn class(&self) -> Option<String> {
        self.id.clone()
    }

    /// Gets the vendor (manufacturer) name for this component.
    #[tracing::instrument(skip(self))]
    pub fn vendor_id(&self) -> Option<String> {
        self.vendor_id.clone()
    }

    /// Returns any status info about this component.
    #[tracing::instrument(skip(self))]
    pub fn status(&self) -> Option<ComponentStatus> {
        self.status.clone()
    }

    /// Gets this component's specific description.
    ///
    /// These contain info that describes the specifications of a component.
    #[tracing::instrument(skip(self))]
    pub fn desc(&self) -> ComponentDescription {
        self.desc.clone()
    }
}

/// The bus a component is on.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub enum ComponentBus {
    Pci,
    Pcie,
    Usb,
    Sys,
    Ps2,
    Serial,
    Eisa,
    Fpci,
    Agp,

    // mostly storage stuff
    Nvme,
    Scsi,
    Ide,

    Other(String),
    Unknown,
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

    /// Describes a power supply, like a battery or AC adapter.
    PowerSupplyDescription(PowerSupplyDescription),

    /// About some storage device.
    StorageDescription(StorageDescription),

    /// About a network interface device.
    NicDescription(NicDescription),

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

    #[tracing::instrument(skip(path))]
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

    #[tracing::instrument]
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

    #[tracing::instrument(skip(wmi))]
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

    #[tracing::instrument(skip(wmi))]
    /// checks for "pnp" devices with a prefix
    pub(super) async fn get_pnp_with_did_prefix(
        wmi: WMIConnection,
        prefix: &str,
    ) -> GhrResult<Vec<ComponentInfo>> {
        let query = get_pnp(wmi).await;
        pnp_filter_did_prefix(query?, prefix).await
    }

    #[tracing::instrument(skip(query))]
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
        #[tracing::instrument(skip(self))]
        fn string_from_variant(&self) -> Option<String> {
            if let Variant::String(s) = self {
                Some(s.clone())
            } else {
                None
            }
        }

        #[tracing::instrument(skip(self))]
        fn u32_from_variant(&self) -> Option<u32> {
            if let Variant::UI4(u) = *self {
                Some(u)
            } else {
                None
            }
        }

        #[tracing::instrument(skip(self))]
        fn u64_from_variant(&self) -> Option<u64> {
            if let Variant::UI8(u) = *self {
                Some(u)
            } else {
                None
            }
        }

        #[tracing::instrument(skip(self))]
        fn bool_from_variant(&self) -> Option<bool> {
            if let Variant::Bool(b) = *self {
                Some(b)
            } else {
                None
            }
        }
    }

    impl VariantInto for Option<&Variant> {
        #[tracing::instrument(skip(self))]
        fn string_from_variant(&self) -> Option<String> {
            if let Some(Variant::String(s)) = self {
                return Some(s.clone());
            }

            None
        }

        #[tracing::instrument(skip(self))]
        fn u32_from_variant(&self) -> Option<u32> {
            if let Some(Variant::UI4(u)) = self {
                return Some(*u);
            }

            // also attempt to get a u64 and cast it.
            // note that this can truncate, but isn't expected to do so outside of tests.
            if let Some(Variant::UI8(u)) = self {
                #[cfg(not(test))]
                tracing::warn!(
                    "Casting `u64` to `u32` outside of tests. This may truncate the value!"
                );
                return Some(*u as u32);
            }

            // or a u16. this is a safe cast on all supported platforms.
            if let Some(Variant::UI2(u)) = self {
                return Some(*u as u32);
            }

            None
        }

        #[tracing::instrument(skip(self))]
        fn u64_from_variant(&self) -> Option<u64> {
            if let Some(Variant::UI8(u)) = self {
                return Some(*u);
            }

            None
        }

        #[tracing::instrument(skip(self))]
        fn bool_from_variant(&self) -> Option<bool> {
            if let Some(Variant::Bool(b)) = self {
                return Some(*b);
            }

            None
        }
    }
}
