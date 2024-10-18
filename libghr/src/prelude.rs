pub(crate) mod internal {
    pub use crate::{
        error::{GhrError, GhrResult},
        report::{
            components::{ComponentBus, ComponentDescription, ComponentInfo, ComponentStatus},
            machine::MachineInfo,
            os::OperatingSystemInfo,
            system_config::SystemConfInfo,
            Report,
        },
    };
}

pub use crate::report::Report;
