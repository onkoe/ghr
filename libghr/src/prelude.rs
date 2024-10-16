pub(crate) mod internal {
    pub use crate::{
        error::{GhrError, GhrResult},
        report::{
            components::ComponentInfo, machine::MachineInfo, os::OperatingSystemInfo,
            system_config::SystemConfInfo, Report,
        },
    };
}

pub use crate::report::Report;
