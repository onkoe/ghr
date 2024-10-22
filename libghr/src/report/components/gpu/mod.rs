use crate::prelude::internal::*;

/// A description for the GPU component of a computer.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct GpuDescription {
    /// Clock speed, in MHz.
    clock_speed: Option<u32>,

    /// Available video memory, in MiB.
    video_memory: Option<u32>,
}

/// Gets information about the system's GPU(s).
pub async fn gpu() -> GhrResult<Vec<ComponentInfo>> {
    todo!()
}
