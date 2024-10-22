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

    // macro that exports to typescript bindings.
    // this prevents me from wasting 80 years doing hand-rolled serialization
    pub use ts_rs::TS as TypeScript;
}

pub use crate::report::Report;
