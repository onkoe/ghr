use std::path::Path;

use crate::prelude::internal::*;

#[tracing::instrument]
pub(super) async fn gpu(gpu: &Path) -> GhrResult<ComponentInfo> {
    // grab some id info about the gpu
    let (id, vendor_id, class) = tokio::join! {
        sysfs_value::<String>(gpu.join("device")),
        sysfs_value::<String>(gpu.join("vendor")),
        sysfs_value::<String>(gpu.join("class")),
    };

    // and now some device info
    let (vram, clock_speed, memory_speed) = tokio::join! {
        sysfs_value::<u64>(gpu.join("mem_info_vram_total")),
        sysfs_value::<u64>(gpu.join("hwmon/hwmon0/freq1_input")),
        sysfs_value::<u64>(gpu.join("hwmon/hwmon0/freq2_input")),
    };

    // convert useless io errors to option
    let (id, vendor_id, class) = (id.ok(), vendor_id.ok(), class.ok());
    let (video_memory, clock_speed, video_memory_speed) =
        (vram.ok(), clock_speed.ok(), memory_speed.ok());

    // map units to mibiunits
    let video_memory = video_memory.map(unit_to_mibiunits);
    let clock_speed =
        clock_speed
            .map(unit_to_mibiunits)
            .and_then(|cs| if cs == 0 { None } else { Some(cs) });
    let video_memory_speed = video_memory_speed.map(unit_to_mibiunits);

    // create the device
    let gpu_info = GpuDescription {
        clock_speed,
        video_memory,
        video_memory_speed,
    };

    Ok(ComponentInfo {
        bus: ComponentBus::Pci,
        id,
        class,
        vendor_id,
        status: None,
        desc: ComponentDescription::GpuDescription(gpu_info),
    })
}
