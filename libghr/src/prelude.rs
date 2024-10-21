pub(crate) mod internal {
    #![allow(unused)]
    pub use crate::{
        error::{GhrError, GhrResult},
        report::{
            components::cpu::{Cache, Frequency},
            components::{
                ComponentBus, ComponentDescription, ComponentInfo, ComponentStatus, Removability,
            },
            machine::MachineInfo,
            os::OperatingSystemInfo,
            system_config::SystemConfInfo,
            Report,
        },
    };
}

pub use crate::report::Report;
