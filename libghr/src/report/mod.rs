pub mod components;
pub mod machine;
pub mod os;
pub mod system_config;

use machine::MachineIdentifier;

use crate::prelude::internal::*;

#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Report {
    os: OperatingSystemInfo,
    machine: MachineInfo,

    /// Information about each hardware component.
    components: Vec<ComponentInfo>,

    /// A general system report on installed software, drivers, and other
    /// system configuration elements.
    sys_conf: SystemConfInfo,
}

impl Report {
    /// Attempts to assemble a new `Report`.
    pub async fn new() -> Result<Self, GhrError> {
        Ok(Self {
            os: Self::os_info()?,
            machine: MachineInfo::new(MachineIdentifier::new_true()?), // TODO: allow MachId::Random

            components: components::get_components().await?,

            sys_conf: system_config::SystemConfInfo {},
        })
    }
}
