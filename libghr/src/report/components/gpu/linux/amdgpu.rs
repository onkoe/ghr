use std::path::Path;

use crate::prelude::internal::*;

#[tracing::instrument]
pub(super) async fn gpu(gpu: &Path) -> GhrResult<ComponentInfo> {
    let gpu = gpu.join("device");

    // grab some id info about the gpu
    let (id, vendor_id, class) = futures::join! {
        sysfs_value_opt::<String>(gpu.join("device")),
        sysfs_value_opt::<String>(gpu.join("vendor")),
        sysfs_value_opt::<String>(gpu.join("class")),
    };

    // turn them into human-readable strings, if possible
    let (class, (id, vendor_id)) = (
        convert_to_pci_class(class),
        convert_to_pci_names(id, vendor_id),
    );

    // and now some device info
    let (video_memory, clock_speed, video_memory_speed) = futures::join! {
        sysfs_value_opt::<u64>(gpu.join("mem_info_vram_total")),
        gpu_clock(&gpu),
        gpu_mem_clock(&gpu)
    };

    // map units to mibiunits
    let video_memory = video_memory.map(unit_to_mibiunits);

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn amdgpu_linux() {
        logger();

        let path = amdgpu_path();
        _ = gpu(&path).await.unwrap();
    }

    #[tokio::test]
    async fn amdgpu_linux_device_ids() {
        logger();

        let path = amdgpu_path();
        let info = gpu(&path).await.unwrap();

        // class
        assert_eq!(
            "Display controller (VGA compatible controller)",
            info.class.unwrap(),
            "device class"
        );

        // vendor
        assert_eq!(
            "Advanced Micro Devices, Inc. [AMD/ATI]",
            info.vendor_id.unwrap(),
            "device vendor"
        );

        // device
        assert_eq!(
            "Navi 22 [Radeon RX 6700/6700 XT/6750 XT / 6800M]",
            info.id.unwrap(),
            "device name"
        );
    }

    #[tokio::test]
    async fn amdgpu_linux_device_specs() {
        logger();

        let path = amdgpu_path();
        let info = gpu(&path).await.unwrap();

        // make sure we saw a gpu here
        let ComponentDescription::GpuDescription(specs) = info.desc else {
            panic!("expected a gpu description, got: {:?}", info.desc);
        };

        // vram amount
        assert_eq!(12272, specs.video_memory.unwrap(), "total vram");

        // clock speed
        assert_eq!(2880, specs.clock_speed.unwrap(), "clock speed");

        // vram clock speed
        assert_eq!(1124, specs.video_memory_speed.unwrap(), "vram clock");
    }

    #[tokio::test]
    async fn amdgpu_linux_clock() {
        logger();

        let path = amdgpu_path();
        let clk = gpu_clock(&path.join("device")).await.unwrap();

        assert_eq!(clk, 2880, "clock speed func");
    }

    #[tokio::test]
    async fn amdgpu_linux_memory_clock() {
        logger();

        let path = amdgpu_path();
        let clk = gpu_mem_clock(&path.join("device")).await.unwrap();

        assert_eq!(clk, 1124, "mem speed func");
    }

    #[tracing::instrument]
    fn amdgpu_path() -> PathBuf {
        let root = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(format!(
            "{root}/tests/assets/linux/sysfs/sys/class/drm/card1"
        ))
    }
}
