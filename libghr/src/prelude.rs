pub(crate) mod internal {
    #![allow(unused)]
    pub(crate) use crate::{
        error::{GhrError, GhrResult},
        report::{
            components::cpu::{Cache, CpuDescription, Frequency},
            components::gpu::GpuDescription,
            components::nic::NicDescription,
            components::psu::PowerSupplyDescription,
            components::ram::RamDescription,
            components::storage::{
                StorageConnector, StorageDescription, StorageKind, StorageUsage,
            },
            components::{
                ComponentBus, ComponentDescription, ComponentInfo, ComponentStatus, Removability,
            },
            machine::MachineInfo,
            os::OperatingSystemInfo,
            system_config::{
                sleep::{Sleep, SleepMode},
                SystemConfInfo,
            },
            util::unit_to_mibiunits,
            Report,
        },
    };

    // re-export the `wmi` helpers on windows
    #[cfg(target_os = "windows")]
    pub(crate) use crate::report::components::windows::{get_wmi, VariantInto};

    // re-export `sysfs` helper on linux
    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub(crate) use crate::report::components::pci::linux::{
        convert_to_pci_class, convert_to_pci_names,
    };
    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub(crate) use crate::report::util::linux::{sysfs_value, sysfs_value_opt, Civ};

    // export logger creating fn for unit tests
    #[cfg(test)]
    pub(crate) use crate::report::util::logger;

    // macro that exports to typescript bindings.
    // this prevents me from wasting 80 years doing hand-rolled serialization
    pub use ts_rs::TS as TypeScript;
}

/// this module's re-exports are, themselves, re-exported by the root of this
/// library crate!
///
/// that makes them available from the crate's root, so instead of importing
///  `libghr::prelude::Thing`, you just grab `libghr::Thing`.
pub(super) mod public {
    pub use crate::{
        error::{GhrError, GhrResult},
        report::Report,
    };
}
