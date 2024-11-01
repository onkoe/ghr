use crate::prelude::internal::*;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

// official nvidia driver, NOT for `nouveau`
mod nvidia;

/// A description for the GPU component of a computer.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct GpuDescription {
    /// Clock speed, in MHz.
    pub clock_speed: Option<u32>,

    /// Available video memory, in MiB.
    pub video_memory: Option<u32>,

    /// Video memory clock speed, in MHz.
    pub video_memory_speed: Option<u32>,
}

#[tracing::instrument]
/// Gets information about the system's GPU(s).
pub async fn get() -> GhrResult<Vec<ComponentInfo>> {
    let mut gpus = Vec::new();

    // only run if the platform has nvidia
    #[cfg(target_os = "windows")]
    if let Ok(mut nvidia_gpus) = nvidia::get().await {
        gpus.append(&mut nvidia_gpus);
    }

    #[cfg(target_os = "linux")]
    if let Ok(mut linux_gpus) = linux::get().await {
        gpus.append(&mut linux_gpus);
    }

    #[cfg(target_os = "windows")]
    if let Ok(mut windows_gpus) = windows::get().await {
        gpus.append(&mut windows_gpus);
    }

    Ok(gpus)
}
