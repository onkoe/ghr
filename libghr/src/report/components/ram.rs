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

#[tracing::instrument]
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

#[tracing::instrument]
#[cfg(target_os = "windows")]
pub async fn ram() -> GhrResult<Vec<ComponentInfo>> {
    use std::collections::HashMap;
    use wmi::Variant;

    // access windows' wmi
    let wmi_connection = get_wmi()?;

    // query `wmi` for memory modules
    let query: Vec<HashMap<String, Variant>> = wmi_connection
        .async_raw_query("SELECT * FROM Win32_PhysicalMemory")
        .await
        .map_err(|e| {
            tracing::error!("Failed to query Windows for memory modules.");
            GhrError::ComponentInfoInaccessible(e.to_string())
        })?;

    // map them into components
    let all_ram = query.into_iter().map(|ram| {
        // grab module info
        let configured_clock_speed = ram.get("ConfiguredClockSpeed").u32_from_variant();
        let configured_voltage = ram.get("ConfiguredVoltage").u32_from_variant();
        let removable = match ram.get("Removable").bool_from_variant() {
            Some(true) => Some(Removability::Removable),
            Some(false) => Some(Removability::NonRemovable),
            None => None,
        };
        let total_phsyical_memory = ram.get("Capacity").u64_from_variant();

        // name + vendor
        let id = ram.get("Name").string_from_variant();
        let vendor_id = ram.get("Manufacturer").string_from_variant();

        // create a ram struct
        let memory_info = RamDescription {
            total_phsyical_memory,
            configured_clock_speed,
            configured_voltage,
            removable,
        };

        // make the component
        ComponentInfo {
            bus: ComponentBus::Sys,
            id,
            class: None,
            vendor_id,
            status: None,
            desc: ComponentDescription::RamDescription(memory_info),
        }
    });

    // return the list
    Ok(all_ram.collect())
}
