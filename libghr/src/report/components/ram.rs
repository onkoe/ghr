//! random access memory information

use crate::prelude::internal::*;

/// For the system memory.
///
/// Some info is unavailable since some machines lack the ability to share
/// per-stick info.
///
/// Also, all values are in bytes.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct RamDescription {
    /// The total amount of physical memory.
    pub total_phsyical_memory: Option<u64>,

    /// The configured clock speed of this module, in MHz.
    pub configured_clock_speed: Option<u32>,

    /// The configured voltage of this module, in mW.
    pub configured_voltage: Option<u32>,

    /// Whether or not the module is removable.
    pub removable: Option<Removability>,
}

#[cfg(target_os = "linux")]
pub async fn ram() -> GhrResult<Vec<ComponentInfo>> {
    use procfs::{Current, FromRead, Meminfo};

    let meminfo = Meminfo::from_file(Meminfo::PATH)
        .map_err(|e| GhrError::ComponentInfoInaccessible(e.to_string()))?;

    let total_mem = meminfo.mem_total;

    Ok(vec![ComponentInfo {
        bus: ComponentBus::Sys,
        id: None,
        class: None,
        vendor_id: None,
        status: None,
        desc: ComponentDescription::RamDescription(RamDescription {
            total_phsyical_memory: Some(total_mem),
            configured_clock_speed: None,
            configured_voltage: None,
            removable: None,
        }),
    }])
}

#[cfg(target_os = "windows")]
pub async fn ram() -> GhrResult<Vec<ComponentInfo>> {
    // access windows' wmi
    use super::windows::get_wmi;
    let wmi_connection = get_wmi()?;

    // make a struct describing phsyical memory (see `Win32_PhysicalMemory`)
    #[derive(serde::Deserialize, Debug)]
    #[allow(non_camel_case_types, non_snake_case, unused)]
    struct Win32_PhysicalMemory {
        pub Capacity: Option<u64>,
        pub ConfiguredClockSpeed: Option<u32>,
        pub ConfiguredVoltage: Option<u32>,
        pub Manufacturer: Option<String>,
        pub Name: Option<String>,
        pub Removable: Option<bool>,

        // we don't use these, but the serialization fails without them for
        // some reason...
        pub Attributes: Option<u32>,
        pub BankLabel: Option<String>,
        pub Caption: Option<String>,
        pub CreationClassName: Option<String>,
        pub DataWidth: Option<u16>,
        pub Description: Option<String>,
        pub DeviceLocator: Option<String>,
        pub FormFactor: Option<u16>,
        pub HotSwappable: Option<bool>,
        pub InstallDate: Option<String>,
        pub InterleaveDataDepth: Option<u16>,
        pub InterleavePosition: Option<u32>,
        pub MaxVoltage: Option<u32>,
        pub MemoryType: Option<u16>,
        pub MinVoltage: Option<u32>,
        pub Model: Option<String>,
        pub OtherIdentifyingInfo: Option<String>,
        pub PartNumber: Option<String>,
        pub PositionInRow: Option<u32>,
        pub PoweredOn: Option<bool>,
        pub Replaceable: Option<bool>,
        pub SerialNumber: Option<String>,
        pub SKU: Option<String>,
        pub SMBIOSMemoryType: Option<u32>,
        pub Speed: Option<u32>,
        pub Status: Option<String>,
        pub Tag: Option<String>,
        pub TotalWidth: Option<u16>,
        pub TypeDetail: Option<u16>,
        pub Version: Option<String>,
    }

    // look for memory modules
    let query: Result<Vec<Win32_PhysicalMemory>, wmi::WMIError> =
        wmi_connection.async_query().await;

    // make sure the query was successful
    let memory = match query {
        Ok(memory) => memory,
        Err(e) => {
            tracing::error!("Failed to query Windows for memory modules.");
            return Err(GhrError::ComponentInfoInaccessible(e.to_string()));
        }
    };

    Ok(memory
        .into_iter()
        .map(|mem| {
            let configured_clock_speed = match mem.ConfiguredClockSpeed {
                Some(0) => None,
                other => Some(other),
            }
            .flatten();

            let configured_voltage = match mem.ConfiguredVoltage {
                Some(0) => None,
                other => Some(other),
            }
            .flatten();

            let removable = match mem.Removable {
                Some(true) => Some(Removability::Removable),
                Some(false) => Some(Removability::NonRemovable),
                None => None,
            };

            ComponentInfo {
                bus: ComponentBus::Sys,
                id: mem.Name,
                class: None,
                vendor_id: mem.Manufacturer,
                status: None,
                desc: ComponentDescription::RamDescription(RamDescription {
                    total_phsyical_memory: mem.Capacity,
                    configured_clock_speed,
                    configured_voltage,
                    removable,
                }),
            }
        })
        .collect::<Vec<ComponentInfo>>())
}
