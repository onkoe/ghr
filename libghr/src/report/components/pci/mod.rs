//! pci

use crate::prelude::internal::*;

#[cfg(target_os = "linux")]
pub(crate) mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[tracing::instrument]
pub async fn get() -> GhrResult<Vec<ComponentInfo>> {
    #[cfg(target_os = "windows")]
    return windows::get().await;

    #[cfg(target_os = "linux")]
    return linux::get().await;
}
