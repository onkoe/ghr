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
pub async fn cpu(_system: &sysinfo::System) -> GhrResult<Vec<ComponentInfo>> {
    use crate::report::components::windows::get_wmi;
    use std::ffi::CString;

    // connect to the windows stuff
    let wmi_connection = get_wmi()?;

    // make a container for the cpu info
    #[allow(non_snake_case)]
    #[derive(serde::Deserialize)]
    struct CpuInfo {
        Manufacturer: CString,
        MaxClockSpeed: u32,
        Name: CString,
        NumberOfCores: u32,
        //
        // cache stuff
        //
        // L2CacheSize: u32,
        // L2CacheSpeed: u32,
        // L3CacheSize: u32,
        // L3CacheSpeed: u32,
    }

    // grab info about cpu
    let res: Result<Vec<CpuInfo>, wmi::WMIError> = wmi_connection.async_query().await;

    let info = match res {
        Ok(cpu_info) => cpu_info,
        Err(e) => {
            tracing::error!("Failed to get information for this CPU.");
            return Err(GhrError::ComponentInfoInaccessible(e.to_string()));
        }
    };

    Ok(info
        .into_iter()
        .map(|cpu| ComponentInfo {
            bus: ComponentBus::Sys,
            id: cpu.Name.to_str().ok().map(|s| s.to_string()),
            class: None,
            vendor_id: cpu.Manufacturer.to_str().ok().map(|s| s.to_string()),
            status: None,
            desc: ComponentDescription::CpuDescription {
                clock_speed: Some(f64::from(cpu.MaxClockSpeed) / 1000_f64),
                core_ct: Some(cpu.NumberOfCores),

                // TODO: we have this info but i'm too lazy to parse it rn
                // see: https://dmtf.org/sites/default/files/standards/documents/DSP0134_3.2.0.pdf,
                //      page 65.
                cache: None,
            },
        })
        .collect::<Vec<_>>())
}
