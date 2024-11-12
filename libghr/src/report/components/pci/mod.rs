//! pci

use crate::prelude::internal::*;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[tracing::instrument]
pub async fn get() -> GhrResult<Vec<ComponentInfo>> {
    #[cfg(target_os = "windows")]
    return windows::get().await;

    #[cfg(any(target_os = "linux", target_os = "android"))]
    return linux::get().await;
}
