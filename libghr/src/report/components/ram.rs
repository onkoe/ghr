//! random access memory information

use crate::prelude::internal::*;

#[cfg(target_os = "linux")]
pub async fn ram() -> GhrResult<ComponentInfo> {
    use procfs::{Current, FromRead, Meminfo};

    let meminfo = Meminfo::from_file(Meminfo::PATH)
        .map_err(|e| GhrError::ComponentInfoInaccessible(e.to_string()))?;

    let total_mem = meminfo.mem_total;

    Ok(ComponentInfo {
        bus: ComponentBus::Sys,
        id: None,
        class: None,
        vendor_id: None,
        status: None,
        desc: ComponentDescription::RamDescription {
            total_phsyical_memory: total_mem,
        },
    })
}
