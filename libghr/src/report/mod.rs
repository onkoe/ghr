pub mod bus;
pub mod components;
pub mod machine;
pub mod os;
pub mod system_config;
pub(crate) mod util;

use machine::MachineIdentifier;

use crate::prelude::internal::*;

#[derive(Clone, Debug, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, TypeScript)]
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
        let (os, machine, initial_components) = tokio::join! {
            Self::os_info(),
            MachineInfo::new(MachineIdentifier::new_random()),
            bus::devices(),
        };

        Ok(Self {
            os: os?,
            machine, // TODO: use the real one

            components: components::get_components(&mut initial_components?).await?,

            sys_conf: system_config::SystemConfInfo {},
        })
    }
}
