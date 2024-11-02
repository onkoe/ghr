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
    pub os: OperatingSystemInfo,
    pub machine: MachineInfo,

    /// Information about each hardware component.
    pub components: Vec<ComponentInfo>,

    /// A general system report on installed software, drivers, and other
    /// system configuration elements.
    pub sys_conf: SystemConfInfo,
}

impl Report {
    #[tracing::instrument]
    /// Attempts to assemble a new `Report`.
    pub async fn new() -> Result<Self, GhrError> {
        // get the machine identifier (used to uniquely identify the report)
        let machine_id = MachineIdentifier::new_random();

        let (os, machine, components, sys_conf) = tokio::join! {
            Self::os_info(),
            MachineInfo::new(machine_id),
            components::get_components(),
            SystemConfInfo::new()
        };

        tracing::debug!("Finished report creation!");

        Ok(Self {
            os: os?,
            machine, // TODO: use the real one
            components: components?,
            sys_conf,
        })
    }

    /// Returns the CPUs attached to this report.
    #[tracing::instrument(skip(self))]
    pub fn cpus(&self) -> Vec<ComponentInfo> {
        self.components
            .clone()
            .into_iter()
            .flat_map(|cmp| {
                if let ComponentDescription::CpuDescription(_) = &cmp.desc() {
                    Some(cmp)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the memory modules attached to this report.
    #[tracing::instrument(skip(self))]
    pub fn memory(&self) -> Vec<ComponentInfo> {
        self.components
            .clone()
            .into_iter()
            .flat_map(|cmp| {
                if let ComponentDescription::RamDescription(_) = &cmp.desc() {
                    Some(cmp)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the GPUs attached to this report.
    #[tracing::instrument(skip(self))]
    pub fn gpus(&self) -> Vec<ComponentInfo> {
        self.components
            .clone()
            .into_iter()
            .flat_map(|cmp| {
                if let ComponentDescription::GpuDescription(_) = &cmp.desc() {
                    Some(cmp)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the power supplies attached to this report.
    #[tracing::instrument(skip(self))]
    pub fn psus(&self) -> Vec<ComponentInfo> {
        self.components
            .clone()
            .into_iter()
            .flat_map(|cmp| {
                if let ComponentDescription::PowerSupplyDescription(_) = &cmp.desc() {
                    Some(cmp)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns any storage devices attached to this report.
    #[tracing::instrument(skip(self))]
    pub fn storage(&self) -> Vec<ComponentInfo> {
        self.components
            .clone()
            .into_iter()
            .flat_map(|cmp| {
                if let ComponentDescription::StorageDescription(_) = &cmp.desc() {
                    Some(cmp)
                } else {
                    None
                }
            })
            .collect()
    }
}
