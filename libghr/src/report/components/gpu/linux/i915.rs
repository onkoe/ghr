use std::path::Path;

use crate::prelude::internal::*;

#[tracing::instrument]
pub(super) async fn gpu(gpu: &Path) -> GhrResult<ComponentInfo> {
    // grab some id info about the gpu
    let (id, vendor_id, class) = tokio::join! {
        sysfs_value_opt::<String>(gpu.join("device/device")),
        sysfs_value_opt::<String>(gpu.join("device/vendor")),
        sysfs_value_opt::<String>(gpu.join("device/class")),
    };

    // grab the clock speed, map to mhz
    let clock_speed = sysfs_value_opt::<u64>(gpu.join("gt_max_freq_mhz"))
        .await
        .and_then(|cs| if cs == 0 { None } else { Some(cs as u32) });

    // make the device
    let gpu_info = GpuDescription {
        clock_speed,
        video_memory: None, // FIXME: little info on the internet about these.
        video_memory_speed: None, // -> look at the source code?
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
