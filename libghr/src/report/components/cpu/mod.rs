//! cpu info

use crate::prelude::internal::*;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

/// About the central processing unit (CPU).
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct CpuDescription {
    /// The CPU's clock speed in MHz. (ex: 3400 MHz)
    pub clock_speed: Frequency,

    /// The CPU's core count.
    pub core_ct: Option<u32>,

    /// Information about the CPU's cache.
    pub cache: Option<Vec<Cache>>,

    /// Information about each CPU core.
    pub cores: Option<Vec<Core>>,
}

/// One of many physical processor cores.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
pub struct Core {
    pub cache: Option<Vec<Cache>>,
    pub speeds: Frequency,
}

/// Core frequencies in MHz.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
pub struct Frequency {
    pub min: Option<u32>,
    pub max: Option<u32>,
}

/// Info about the CPU's cache.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub enum Cache {
    L1 { size: u32, speed: Option<u32> },
    L2 { size: u32, speed: Option<u32> },
    L3 { size: u32, speed: Option<u32> },
}

#[tracing::instrument]
/// Gets info about the CPU.
pub async fn get() -> GhrResult<Vec<ComponentInfo>> {
    #[cfg(target_os = "windows")]
    return windows::get().await;

    #[cfg(any(target_os = "linux", target_os = "android"))]
    return linux::get().await;
}
