use crate::prelude::internal::*;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

/// A description for a network card or similar device.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct NicDescription {
    /// The known maximum speed of the card, in Mbps.
    pub max_speed: Option<u32>,

    /// The "maximum transfer unit" of a network interface card.
    pub mtu: Option<u32>,
}

/// Finds and returns info about network devices on the system.
#[tracing::instrument]
pub async fn get() -> GhrResult<Vec<ComponentInfo>> {
    #[cfg(target_os = "windows")]
    return windows::get().await;

    #[cfg(target_os = "linux")]
    return linux::get().await;
}
