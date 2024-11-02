use crate::prelude::internal::*;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

/// Information about the computer's supported sleep states.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct Sleep {
    /// Idle standby, emulated by the operating system.
    ///
    /// This can become `s0ix` if the hardware supports it.
    s0: SleepMode,
    /// Traditional "standby" suspend.
    s1: SleepMode,
    /// Like standby, but the processor's context also needs to be restored.
    ///
    /// This is most common on modern machines.
    s2: SleepMode,
    /// Sometimes known as "modern standby" or "Always On Always Connected",
    /// "AOAC".
    ///
    /// This allows the system to turn off components as needed, allowing for
    /// intelligent wake-up.
    s0ix: SleepMode,
    /// "Suspend-to-RAM". RAM is powered to maintain pre-sleeping state.
    s3: SleepMode,
    /// "Suspend-to-disk", or "hibernation". Everything is turned off and the
    /// system may fully power down.
    s4: SleepMode,
}

impl Default for Sleep {
    #[tracing::instrument]
    fn default() -> Self {
        Self {
            s0: SleepMode::Unknown,
            s1: SleepMode::Unknown,
            s2: SleepMode::Unknown,
            s0ix: SleepMode::Unknown,
            s3: SleepMode::Unknown,
            s4: SleepMode::Unknown,
        }
    }
}

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

impl From<bool> for SleepMode {
    fn from(value: bool) -> Self {
        if value {
            Self::Supported
        } else {
            Self::Unsupported
        }
    }
}

/// Gets information about supported ACPI sleep states.
#[tracing::instrument]
pub async fn get() -> Sleep {
    #[cfg(target_os = "linux")]
    return linux::get().await;

    #[cfg(target_os = "windows")]
    return windows::get().await;
}
