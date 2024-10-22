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
    vendor: String,
    /// This machine's model number.
    model: String,

    /// A hash that uniquely identifies this computer.
    ///
    /// Note: This might be randomly-generated if the user doesn't want to send
    /// this info around.
    hash: MachineIdentifier,
}

impl MachineInfo {
    pub fn new(machine_id: MachineIdentifier) -> Self {
        // TODO: fill these with stuff
        let vendor = "".into();
        let model = "".into();

        Self {
            vendor,
            model,
            hash: machine_id,
        }
    }
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
    /// Computes a true identifier for this machine.
    pub fn new_true() -> GhrResult<Self> {
        Ok(Self::True(hash::make_hash()?))
    }

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
    #[cfg(target_os = "linux")]
    pub fn machine_info() -> MachineInfoReturnType {
        Ok(MachineInfo::new(MachineIdentifier::new_true()?))
    }
}
