use async_fs::{self, read_link, DirEntry};
use futures::TryStreamExt;
use regex::Regex;

use crate::prelude::internal::*;

/// a static path where gpus reprs are placed by the kernel/drivers
const GPU_LISTING: &str = "/sys/class/drm";

mod amdgpu;
mod generic;
mod i915;

#[tracing::instrument]
pub(super) async fn get() -> GhrResult<Vec<ComponentInfo>> {
    // grab devices from the system
    let devices = devices().await?;

    let mut list = vec![];

    // we want to loop over each gpu. getting data for a gpu can be different
    // for many reasons, particularly per driver.
    for device in devices {
        let path = device.path();
        let path_str = path.display();

        // get the driver for the gpu
        let Ok(driver) = find_driver(device).await else {
            tracing::warn!("Failed to find driver for device at `{path_str}`.",);
            continue;
        };

        // based on the driver, pick an implementation to use
        tracing::debug!("parsing gpu with `{driver}` driver at `{path_str}`...",);
        let info: Result<ComponentInfo, GhrError> = match driver.to_ascii_lowercase().as_str() {
            "amdgpu" => amdgpu::gpu(&path).await,
            "i915" => i915::gpu(&path).await,
            "nvidia" => {
                // ignore nvidia gpus; they're located with `nvml`, not `sysfs`
                //
                // (see `super::nvidia` for more info)
                continue;
            }
            _ => {
                tracing::warn!(
                    "No information about this generic device. An \
                empty output will result for this entry. (driver: {driver}, path: {path_str})"
                );
                generic::gpu(&path).await
            }
        };

        // make sure everything went well
        let Ok(info) = info else {
            tracing::warn!("Couldn't parse info for GPU at `{path_str}`.");
            continue;
        };

        list.push(info);
    }

    Ok(list)
}

#[tracing::instrument]
/// gets the gpus on the system.
///
/// do not export this!
async fn devices() -> GhrResult<Vec<DirEntry>> {
    // to check if `drm/cardN` directories are the "main" ones
    let regex = Regex::new(r#"^card\d+$"#).map_err(|e| {
        tracing::error!("Regex creation failed!");
        GhrError::RegexCreationFailure(e.to_string())
    })?;

    // grab directory
    let mut entries = async_fs::read_dir(GPU_LISTING).await.map_err(|e| {
        tracing::error!(
            "Failed to read the `{GPU_LISTING}` directory on Linux, which should be static."
        );
        GhrError::ComponentInfoInaccessible(e.to_string())
    })?;

    // grab only the entries we want
    let mut gpus = Vec::new();
    while let Ok(Some(en)) = entries.try_next().await {
        // we only want directories that look like `cardN`.
        let name = en.file_name();

        if regex.is_match(&name.to_string_lossy()) {
            gpus.push(en);
        }
    }

    Ok(gpus)
}

#[tracing::instrument(skip(device))]
/// finds the driver for a device listing in `/sys/class/drm/cardN`.
async fn find_driver(device: DirEntry) -> GhrResult<String> {
    // first, we want to navigate to the `device` folder
    let path = device.path().join("device");

    // and then open up the `driver` dir, but return the folder it points to
    // since it's a symlink
    let driver_path = read_link(path.join("driver")).await.map_err(|e| {
        tracing::error!("Couldn't find this GPU's driver! The `driver` dir should always be a symlink, but reading it failed. (err: {e})");
        GhrError::ComponentInfoInaccessible(format!("Failed to follow GPU driver symlink. (err: {e}"))
    })?;

    // we only need that dir's name... as a string
    let driver = driver_path
        .file_name()
        .ok_or(GhrError::ComponentInfoInaccessible(
            "Failed to find the GPU driver dir's name".into(),
        ))?
        .to_string_lossy()
        .to_string();

    Ok(driver)
}
