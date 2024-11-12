//! `machine`: Grabs info about the machine manufacturer and model.

use rand::{distributions, Rng as _};

use crate::prelude::internal::*;

mod hash;

pub use hash::Hash;

pub type MachineInfoReturnType = GhrResult<MachineInfo>;

#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct MachineInfo {
    /// The organization that created the machine.
    vendor: Option<String>,
    /// This machine's model number.
    model: Option<String>,

    /// Info about the computer's BIOS.
    bios: BiosInfo,

    /// Info about the machine's chassis.
    chassis: ChassisInfo,

    /// A hash that uniquely identifies this computer.
    ///
    /// Note: This might be randomly-generated if the user doesn't want to send
    /// this info around.
    hash: MachineIdentifier,
}

impl MachineInfo {
    #[tracing::instrument(skip(machine_id))]
    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub async fn new(machine_id: MachineIdentifier) -> Self {
        use std::path::PathBuf;

        // the path to the machine info on linux
        let sysfs_info = PathBuf::from("/sys/devices/virtual/dmi/id");

        // read the vendor + model from `sysfs`
        let vendor = sysfs_value(sysfs_info.join("sys_vendor")).await.ok();
        let model = sysfs_value("product_name").await.ok();

        // chassis info
        let chassis = ChassisInfo {
            kind: sysfs_value::<String>(sysfs_info.join("chassis_type"))
                .await
                .ok(),
            vendor: sysfs_value::<String>(sysfs_info.join("chassis_vendor"))
                .await
                .ok(),
            version: sysfs_value::<String>(sysfs_info.join("chassis_version"))
                .await
                .ok(),
        };

        // bios info
        let bios = BiosInfo {
            vendor: sysfs_value::<String>(sysfs_info.join("bios_vendor"))
                .await
                .ok(),
            version: sysfs_value::<String>(sysfs_info.join("bios_version"))
                .await
                .ok(),
            date: sysfs_value::<String>(sysfs_info.join("bios_date"))
                .await
                .ok()
                .and_then(|date_str| chrono::NaiveDate::parse_from_str(&date_str, "%m/%d/%Y").ok()),
        };

        Self {
            vendor,
            model,
            bios,
            chassis,
            hash: machine_id,
        }
    }

    #[tracing::instrument(skip(machine_id))]
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    pub async fn new(machine_id: MachineIdentifier) -> Self {
        // system vendor + model
        let vendor = None;
        let model = None;

        // chassis info
        let chassis = ChassisInfo {
            kind: None,
            vendor: None,
            version: None,
        };

        // bios info
        let bios = BiosInfo {
            vendor: None,
            version: None,
            date: None,
        };

        Self {
            vendor,
            model,
            bios,
            chassis,
            hash: machine_id,
        }
    }
}

/// Information about the chassis.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct ChassisInfo {
    /// The kind of chassis this machine is inside.
    ///
    /// Examples include "Desktop" or "Tablet".
    pub kind: Option<String>,
    /// The creator of the system chassis.
    pub vendor: Option<String>,
    pub version: Option<String>,
}

/// Information about the system BIOS.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
#[non_exhaustive]
pub struct BiosInfo {
    /// The creator of the system BIOS.
    pub vendor: Option<String>,
    /// The system BIOS version.
    pub version: Option<String>,
    /// The date the BIOS was compiled.
    pub date: Option<chrono::NaiveDate>,
}

/// A unique identifier for each computer.
#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
pub enum MachineIdentifier {
    /// This is a 'true' identifier - it uniquely identifies the machine based
    /// on its hardware.
    True(Hash),
    /// This identifier was randomly generated.
    Random(Hash),
}

impl MachineIdentifier {
    #[tracing::instrument]
    /// Computes a true identifier for this machine.
    pub fn new_true() -> GhrResult<Self> {
        Ok(Self::True(hash::make_hash()?))
    }

    #[tracing::instrument]
    /// Creates a random identifier for this machine.
    pub fn new_random() -> Self {
        let s = rand::thread_rng()
            .sample_iter(distributions::Standard)
            .take(30)
            .collect();

        Self::Random(s)
    }
}

impl Report {
    #[tracing::instrument]
    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub async fn machine_info() -> MachineInfoReturnType {
        Ok(MachineInfo::new(MachineIdentifier::new_true()?).await)
    }
}
