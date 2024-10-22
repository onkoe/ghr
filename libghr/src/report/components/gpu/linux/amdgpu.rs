use super::devices;
use crate::prelude::internal::*;

#[tracing::instrument]
pub(super) async fn gpus() -> GhrResult<Vec<ComponentInfo>> {
    // grab all gpus
    let devices = devices().await?;

    // find info for each gpu
    let mut info = vec![];
    for device in devices {
        let gpu = device.path().join("device");

        let Ok(driver) = sysfs_value::<String>(gpu.join("hwmon/hwmon0/name")).await else {
            tracing::error!("Failed to read GPU driver.");
            return Err(GhrError::ComponentUnsupported(String::from(
                "Failed to read GPU driver.",
            )));
        };

        if driver != "amdgpu" {
            return Err(GhrError::ComponentUnsupported(String::from(
                "You must use an `amdgpu` in this function",
            )));
        }

        // TODO: MOVE THE PRIOR LOGIC TO `super`. ONLY CALL THIS IF A GPU IS FROM AMD!

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

        info.push(ComponentInfo {
            bus: ComponentBus::Pci,
            id,
            class,
            vendor_id,
            status: None,
            desc: ComponentDescription::GpuDescription(gpu_info),
        });
    }

    Ok(info)
}
