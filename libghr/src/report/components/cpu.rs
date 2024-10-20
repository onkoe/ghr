//! cpu info

use crate::prelude::internal::*;

/// Info about the CPU's cache.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum CpuCache {
    L2 { size: u32, speed: u32 },
    L3 { size: u32, speed: u32 },
}

#[cfg(target_os = "linux")]
pub async fn cpu(system: &sysinfo::System) -> GhrResult<Vec<ComponentInfo>> {
    use procfs::{CpuInfo, FromBufRead};
    use std::path::PathBuf;
    use std::{fs::File, io::BufReader};

    // grab info about cpu
    let cpu_info_file = PathBuf::from("/proc/cpuinfo");
    let rdr = BufReader::new(
        File::open(cpu_info_file)
            .map_err(|e| GhrError::ComponentInfoInaccessible(e.to_string()))?,
    );
    let info = CpuInfo::from_buf_read(rdr)
        .map_err(|e| GhrError::ComponentInfoInaccessible(e.to_string()))?;

    // make the speed/core count
    let sysinfo_cpus = system.cpus();
    let core_ct = info.num_cores() as u32;
    let speed = sysinfo_cpus
        .iter()
        .map(|core| core.frequency())
        .max()
        .unwrap_or_else(|| {
            tracing::warn!("CPU didn't report a speed.");
            0
        });

    // general identifiers
    let id = info.model_name(0).map(|s| s.to_string());
    let vendor_id = info.vendor_id(0).map(|s| s.to_string());

    // report any weirdness
    if core_ct == 0 {
        tracing::warn!("CPU reported having zero cores...");
    }

    Ok(vec![ComponentInfo {
        bus: ComponentBus::Sys,
        id,
        class: None, // TODO
        vendor_id,
        status: None,
        desc: ComponentDescription::CpuDescription {
            clock_speed: Some(speed as f64 / 1000_f64),
            core_ct: Some(core_ct),
            cache: None,
        },
    }])
}

#[cfg(target_os = "windows")]
#[tracing::instrument]
pub async fn cpu(_system: &sysinfo::System) -> GhrResult<Vec<ComponentInfo>> {
    use crate::report::components::windows::get_wmi;
    use std::collections::HashMap;
    use wmi::Variant;

    // connect to the windows stuff
    let wmi = get_wmi()?;

    // grab info about cpu
    tracing::debug!("looking for CPUs...");
    let query: Result<Vec<HashMap<String, Variant>>, _> =
        wmi.async_raw_query("SELECT * from Win32_Processor").await;

    // unwrap it
    let query = match query {
        Ok(cpu_info) => cpu_info,
        Err(e) => {
            tracing::error!("Couldn't get CPU information.");
            return Err(GhrError::ComponentInfoInaccessible(e.to_string()));
        }
    };

    // make it into real info
    let mut cpus = Vec::new();
    for cpu in query {
        let name = cpu.get("Name").and_then(|s| {
            if let Variant::String(name) = s {
                Some(name.trim().to_string())
            } else {
                None
            }
        });

        let manufacturer = cpu
            .get("Manufacturer")
            .and_then(|s| {
                if let Variant::String(vendor) = s {
                    Some(vendor)
                } else {
                    None
                }
            })
            .cloned();

        // note: we / this by 1000 to get the ghz
        let speed = cpu.get("MaxClockSpeed").and_then(|s| {
            if let Variant::UI4(clk) = *s {
                Some(f64::from(clk) / 1000_f64)
            } else {
                None
            }
        });

        let number_of_cores = cpu.get("NumberOfCores").and_then(|s| {
            if let Variant::UI4(clk) = *s {
                Some(clk)
            } else {
                None
            }
        });

        cpus.push(ComponentInfo {
            bus: ComponentBus::Sys,
            id: name,
            class: None,
            vendor_id: manufacturer,
            status: None,
            desc: ComponentDescription::CpuDescription {
                clock_speed: speed,
                core_ct: number_of_cores,

                // TODO: we have this info but i'm too lazy to parse it rn
                // see: https://dmtf.org/sites/default/files/standards/documents/DSP0134_3.2.0.pdf,
                //      page 65.
                cache: None,
            },
        })
    }

    tracing::debug!("found {} CPUs!", cpus.len());

    Ok(cpus)
}
