//! psu: power supply and battery info

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

use crate::prelude::internal::*;

/// A desciption of a power supply for the system.
///
/// This includes both batteries and AC adapters.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub enum PowerSupplyDescription {
    Battery {
        /// the tech the battery is based on, like Li-ion
        technology: Option<String>,

        /// the maximum battery capacity, in wh, that the system has observed
        real_capacity_wh: Option<f64>,

        /// the theoretical maximum battery capacity in wh
        theoretical_capacity_wh: Option<f64>,

        /// the number of times the battery has been charged.
        ///
        /// seems like this value can be negative, so using an i32 for now.
        cycle_count: Option<i32>,
    },

    Ac {},
}

#[tracing::instrument]
pub async fn get() -> GhrResult<Vec<ComponentInfo>> {
    #[cfg(target_os = "linux")]
    return linux::get().await;

    #[cfg(target_os = "windows")]
    return windows::get().await;
}
