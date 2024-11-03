use std::collections::HashMap;

use futures::StreamExt;
use wmi::Variant;

use crate::prelude::internal::*;

/// Finds and returns info about network devices on the system.
#[tracing::instrument]
pub(crate) async fn get() -> GhrResult<Vec<ComponentInfo>> {
    let wmi = get_wmi()?;

    // query wmi for network interfaces
    let query: Vec<HashMap<String, Variant>> = wmi
        .async_raw_query("SELECT * from Win32_NetworkAdapter")
        .await
        .map_err(|e| {
            tracing::warn!("Failed to access network interface cards from `wmi`. (err: {e})");
            GhrError::ComponentInfoInaccessible(format!(
                "Failed to access network interface cards from `wmi`. (err: {e})"
            ))
        })?;

    // send the query for parsing
    Ok(all(query).await)
}

/// finds info about the network devices represented by the given `wmi` query.
#[tracing::instrument(skip(query))]
async fn all(query: Vec<HashMap<String, Variant>>) -> Vec<ComponentInfo> {
    // map each into a componentinfo, if one is attainable
    futures::stream::iter(query).filter_map(one).collect().await
}

/// given one serialized entry from `wmi`, parses it into an nic repr.
#[tracing::instrument(skip(fields))]
async fn one(fields: HashMap<String, Variant>) -> Option<ComponentInfo> {
    // find its name and vendor
    let name = fields.get("Name").and_then(|v| v.string_from_variant());
    let vendor = fields
        .get("Manufacturer")
        .and_then(|v| v.string_from_variant());

    // we only want to report if this is a "physical" adapter
    let is_real = fields
        .get("PhysicalAdapter")
        .and_then(|v| v.bool_from_variant())?;

    if !is_real {
        tracing::debug!("Skipped a non-physical adapter. (name: {name:?}, vendor: {vendor:?})");
        return None;
    }

    // specific info: just speed for now. (mtu not supported?)
    let max_speed = fields
        .get("MaxSpeed")
        .and_then(|v| v.u64_from_variant())
        .map(|bits| {
            // we need to convert the bits per second into Mbps.
            // that's just (bits * 1000 = kbits) * 1000 = mbits
            bits * 1000 * 1000
        })
        .and_then(|megs| u32::try_from(megs).ok());

    Some(ComponentInfo {
        bus: ComponentBus::Unknown,
        id: name,
        class: None,
        vendor_id: vendor,
        status: None,
        desc: ComponentDescription::NicDescription(NicDescription {
            max_speed,
            mtu: None,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn check_windows_ax200_wireless() {
        let path = ax200_path();
        let query = serde_json::from_str(&tokio::fs::read_to_string(path).await.unwrap()).unwrap();

        let nics = all(query).await;
        let cmp = nics.first().unwrap();

        // name and vendor
        assert_eq!(cmp.id().unwrap(), "Intel(R) Wi-Fi 6 AX200 160MHz");
        assert_eq!(cmp.vendor_id().unwrap(), "Intel Corporation");

        // we should have a network card description
        let ComponentDescription::NicDescription(desc) = cmp.desc() else {
            panic!("wrong one!");
        };

        // speed is null for this one. it's correct to be none.
        assert!(desc.max_speed.is_none());
    }

    #[tracing::instrument]
    fn ax200_path() -> PathBuf {
        let root = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(root).join("tests/assets/windows/ax200_nic.json")
    }
}
