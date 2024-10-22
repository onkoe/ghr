use crate::prelude::internal::*;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

/// A description for the GPU component of a computer.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct GpuDescription {
    /// Clock speed, in MHz.
    clock_speed: Option<u32>,

    /// Available video memory, in MiB.
    video_memory: Option<u32>,

    /// Video memory clock speed, in MHz.
    video_memory_speed: Option<u32>,
}

/// Gets information about the system's GPU(s).
pub async fn gpu() -> GhrResult<Vec<ComponentInfo>> {
    #[cfg(target_os = "linux")]
    return linux::gpus().await;

    #[cfg(target_os = "windows")]
    return windows::gpus().await;
}
