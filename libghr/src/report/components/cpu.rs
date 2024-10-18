//! cpu info

use crate::prelude::internal::*;

#[cfg(target_os = "linux")]
pub async fn cpu(system: &sysinfo::System) -> GhrResult<ComponentInfo> {
    // grab info about pci devices and construct reprs

    use std::path::PathBuf;

    use procfs::{CpuInfo, FromBufRead};
    use std::{fs::File, io::BufReader};


    let cpu_info_file = PathBuf::from("/proc/cpuinfo");
    let rdr = BufReader::new(
        File::open(cpu_info_file)
            .map_err(|e| GhrError::ComponentInfoInaccessible(e.to_string()))?,
    );
    let info = CpuInfo::from_buf_read(rdr)
        .map_err(|e| GhrError::ComponentInfoInaccessible(e.to_string()))?;

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
    let id = info.model_name(0).unwrap_or("Unknown").to_string();
    let vendor_id = info
        .vendor_id(0)
        .unwrap_or_else(|| {
            tracing::warn!("CPU didn't report a vendor ID.");
            "Unknown"
        })
        .to_string();

    // report any weirdness
    if core_ct == 0 {
        tracing::warn!("CPU reported having zero cores...");
    }

    Ok(ComponentInfo {
        bus: ComponentBus::Sys,
        id,
        class: String::from("CPU"), // TODO
        vendor_id,
        status: ComponentStatus {},
        desc: ComponentDescription::CpuDescription {
            clock_speed: (speed as f64 / 1000_f64),
            core_ct,
        },
    })
}
