#![cfg(target_os = "windows")]

use crate::prelude::internal::*;

#[tracing::instrument]
pub async fn cpu() -> GhrResult<Vec<ComponentInfo>> {
    use crate::report::components::windows::get_wmi;
    use std::collections::HashMap;
    use wmi::Variant;

    // connect to the windows stuff
    let wmi = get_wmi()?;

    // grab info about cpu
    tracing::debug!("looking for CPUs...");
    let query: Result<Vec<HashMap<String, Variant>>, _> =
        wmi.async_raw_query("SELECT * from Win32_Processor").await;

    // unwrap it
    let query = match query {
        Ok(cpu_info) => cpu_info,
        Err(e) => {
            tracing::error!("Couldn't get CPU information.");
            return Err(GhrError::ComponentInfoInaccessible(e.to_string()));
        }
    };

    // make it into real info
    let mut cpus = Vec::new();
    for cpu in query {
        let name = cpu.get("Name").and_then(|s| {
            if let Variant::String(name) = s {
                Some(name.trim().to_string())
            } else {
                None
            }
        });

        let manufacturer = cpu
            .get("Manufacturer")
            .and_then(|s| {
                if let Variant::String(vendor) = s {
                    Some(vendor)
                } else {
                    None
                }
            })
            .cloned();

        let speed = cpu.get("MaxClockSpeed").and_then(|s| {
            if let Variant::UI4(clk) = *s {
                Some(clk)
            } else {
                None
            }
        });

        let number_of_cores = cpu.get("NumberOfCores").and_then(|s| {
            if let Variant::UI4(clk) = *s {
                Some(clk)
            } else {
                None
            }
        });

        cpus.push(ComponentInfo {
            bus: ComponentBus::Sys,
            id: name,
            class: None,
            vendor_id: manufacturer,
            status: None,
            desc: ComponentDescription::CpuDescription {
                clock_speed: Frequency {
                    min: None,
                    max: speed,
                },
                core_ct: number_of_cores,

                // TODO: we have this info but i'm too lazy to parse it rn
                // see: https://dmtf.org/sites/default/files/standards/documents/DSP0134_3.2.0.pdf,
                //      page 65.
                cache: None,
                cores: None,
            },
        })
    }

    tracing::debug!("found {} CPUs!", cpus.len());

    Ok(cpus)
}
