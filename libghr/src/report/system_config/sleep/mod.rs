use crate::prelude::internal::*;

#[cfg(target_os = "linux")]
mod linux;

/// Information about the computer's supported sleep states.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct Sleep {
    /// Traditional "standby" suspend.
    s1: bool,
    /// Like standby, but the processor's context also needs to be restored.
    ///
    /// This is most common on modern machines.
    s2: bool,
    /// Sometimes known as "modern standby" or "s2idle".
    ///
    /// This allows the system to turn off components as needed, allowing for
    /// intelligent wake-up.
    s0ix: bool,
    /// "Suspend-to-RAM". RAM is powered to maintain pre-sleeping state.
    s3: bool,
    /// "Suspend-to-disk", or "hibernation". Everything is turned off and the
    /// system may fully power down.
    s4: bool,

/// Indicates whether or not a sleep state (e.g., "S2") is
/// supported by the computer.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
pub enum SleepMode {
    /// Definitively supported.
    Supported,
    /// Definitively NOT supported.
    Unsupported,
    /// We couldn't collect enough information about whether or not the
    /// system supports this sleep state.
    Unknown,
}
