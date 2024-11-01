use std::collections::HashMap;

use crate::{prelude::internal::*, report::components::gpu::GpuDescription};
use wmi::Variant;

#[tracing::instrument]
/// grabs the system's gpus.
pub(super) async fn get() -> GhrResult<Vec<ComponentInfo>> {
    // first, grab the wmi connection if we need it
    let wmi = get_wmi()?;

    // grab a list of all gpu devices
    let query: Vec<HashMap<String, Variant>> = wmi
        .async_raw_query("SELECT * FROM Win32_VideoController")
        .await
        .map_err(|e| {
            GhrError::ComponentInfoInaccessible(format!(
                "WMI failed to access GPU information for this system. (err: {e}"
            ))
        })?;

    // collect into a list, then return it
    let gpus = query
        .into_iter()
        .map(|gpu| {
            tracing::trace!("Checking GPU...");

            // grab important info
            let id = gpu.get("Name").string_from_variant();
            let vendor_id = None;
            let vram = gpu.get("AdapterRAM").u32_from_variant();

            // make gpu info
            let gpu_desc = GpuDescription {
                clock_speed: None,
                video_memory: vram,
                video_memory_speed: None,
            };

            // yield a ComponentInfo
            ComponentInfo {
                bus: ComponentBus::Pci, // FIXME: we can check
                id,
                vendor_id,   // FIXME: check this too
                class: None, // TODO
                status: None,
                desc: ComponentDescription::GpuDescription(gpu_desc),
            }
        })
        .collect();

    Ok(gpus)
}
