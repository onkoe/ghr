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

/// returns the gpu clock speed in MHz.
#[tracing::instrument]
async fn gpu_clock(gpu: &Path) -> Option<u32> {
    clock(gpu, "pp_dpm_sclk").await
}

/// returns the gpu memory clock speed in MHz.
#[tracing::instrument]
async fn gpu_mem_clock(gpu: &Path) -> Option<u32> {
    clock(gpu, "pp_dpm_mclk").await
}

/// gets some speed in MHz from the given source.
///
/// this is for the `amdgpu` driver only, and it's a helper function.
///
/// (in other words, don't call with weird stuff)
#[tracing::instrument]
async fn clock<S: AsRef<str> + std::fmt::Debug>(gpu: &Path, file: S) -> Option<u32> {
    let file = file.as_ref();
    println!("in clock: {}, {file}", gpu.display());

    let Some(clk_string) = sysfs_value_opt::<String>(gpu.join(file)).await else {
        tracing::warn!("The `{file}` file does not exist for the given GPU.");
        return None;
    };

    // great, it exists! now the file we've got should look like this:
    //
    //  ```
    //  0: 500Mhz *
    //  1: 2880Mhz
    //  ```
    //
    // the last (nth) line has that maximum value we're looking for

    // skip first line
    let Some(clk_line) = clk_string.lines().last() else {
        tracing::warn!("The `{file}` file was blank.");
        return None;
    };

    // split on the space after `n:`, then use the second part of the string
    let Some(clk_str_mhz) = clk_line.trim().split_ascii_whitespace().nth(1) else {
        tracing::warn!("Failed to parse the `MHz` part of the `{file}` file.");
        return None;
    };

    // replace the "Mhz" with nothing
    let clk_str_mhz = clk_str_mhz.replace("Mhz", "");

    // try parsing it into a value
    match clk_str_mhz.trim().parse::<u32>() {
        Ok(clk) => Some(clk),
        Err(e) => {
            tracing::warn!(
                "Failed to parse clock speed as `u32` value! (str: `{clk_str_mhz}`, err: {e}"
            );
            None
        }
    }
}

