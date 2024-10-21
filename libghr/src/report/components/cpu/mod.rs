//! cpu info

use crate::prelude::internal::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

/// One of many physical processor cores.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Core {
    pub cache: Option<Vec<Cache>>,
    pub speeds: Frequency,
}

/// Core frequencies in MHz.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Frequency {
    pub min: Option<u32>,
    pub max: Option<u32>,
}

/// Info about the CPU's cache.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum Cache {
    L1 { size: u32, speed: Option<u32> },
    L2 { size: u32, speed: Option<u32> },
    L3 { size: u32, speed: Option<u32> },
}

/// Gets info about the CPU.
pub async fn cpu() -> GhrResult<Vec<ComponentInfo>> {
    #[cfg(target_os = "windows")]
    return windows::cpu().await;

    #[cfg(target_os = "linux")]
    return linux::cpu().await;
}
