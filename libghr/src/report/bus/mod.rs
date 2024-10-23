//! # bus: finds the buses that components live on
//!
//! This module creates preliminary listings for components all across the
//! system, saving paths alongside very basic component information.
//!
//! In doing so, it allows the library to avoid creating duplicate component
//! listings, unifying information across the system.

use std::path::PathBuf;

use crate::prelude::internal::*;

mod linux;

/// A device listing without specialized information.
///
/// In other words, it only has info about the bus - nothing else.
pub struct InitialDevice {
    /// The listing's path in `/sys/bus`, like `/sys/bus/soc/devices/<device>`.
    path: PathBuf,

    /// A half-filled [`ComponentInfo`]. Does not contain any
    /// component-specific information.
    pub info: ComponentInfo,
}

impl InitialDevice {
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl From<InitialDevice> for ComponentInfo {
    /// This converts the given [`InitialDeviceListing`] into a [`ComponentInfo`].
    ///
    /// Please only use this if either:
    ///
    /// - the device is being completed by a specific component information
    ///   gathering implementation, or
    /// - no implementation is available for the device, so we only have
    ///   generic info
    ///
    /// Note that this discards the path, so the above point is significant.
    fn from(value: InitialDevice) -> Self {
        value.info
    }
}

/// Collects
pub async fn devices() -> GhrResult<Vec<InitialDevice>> {
    todo!()
}
