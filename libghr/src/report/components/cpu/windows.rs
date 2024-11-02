use std::collections::HashMap;

use wmi::Variant;

use crate::prelude::internal::*;

#[tracing::instrument]
pub async fn get() -> GhrResult<Vec<ComponentInfo>> {
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
    Ok(from_wmi_query(query).await)
}

/// takes a wmi query and searches for any cpus it contains.
#[tracing::instrument(skip(query))]
async fn from_wmi_query(query: Vec<HashMap<String, Variant>>) -> Vec<ComponentInfo> {
    let mut cpus = Vec::new();
    for cpu in query {
        let name = cpu
            .get("Name")
            .string_from_variant()
            .map(|s| s.trim().to_string());
        let manufacturer = cpu.get("Manufacturer").string_from_variant();

        let speed = cpu.get("MaxClockSpeed").u32_from_variant();
        let number_of_cores = cpu.get("NumberOfCores").u32_from_variant();

        cpus.push(ComponentInfo {
            bus: ComponentBus::Sys,
            id: name,
            class: None,
            vendor_id: manufacturer,
            status: None,
            desc: ComponentDescription::CpuDescription(CpuDescription {
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
            }),
        })
    }
    tracing::debug!("found {} CPUs!", cpus.len());

    cpus
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[tokio::test]
    async fn check_amd_cpu_windows() {
        // grab the cpu
        let query = amd_cpu_query().await;

        let cpu_list = from_wmi_query(query).await;
        let cpu = cpu_list.first().unwrap();

        // ensure it's got the right name + vendor
        assert_eq!(cpu.id().unwrap(), "AMD Ryzen 7 5800X 8-Core Processor");
        assert_eq!(cpu.vendor_id().unwrap(), "AuthenticAMD");

        // grab specific info
        let ComponentDescription::CpuDescription(desc) = cpu.desc() else {
            panic!("wrong desc");
        };

        // check specific cpu info
        assert_eq!(desc.core_ct.unwrap(), 8);
        assert_eq!(desc.clock_speed.max.unwrap(), 3801);
    }

    #[tracing::instrument]
    async fn amd_cpu_query() -> Vec<HashMap<String, Variant>> {
        let root = env!("CARGO_MANIFEST_DIR");
        let path = PathBuf::from(format!("{root}/tests/assets/windows/cpu.json"));

        // load the query
        serde_json::from_str(&async_fs::read_to_string(path).await.unwrap()).unwrap()
    }
}
