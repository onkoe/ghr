use futures::StreamExt as _;
use wmi::Variant;

use crate::prelude::internal::*;
use std::collections::HashMap;

/// grabs all known linux `block` devices from `sysfs`.
#[tracing::instrument]
pub(crate) async fn get() -> GhrResult<Vec<ComponentInfo>> {
    // grab storage devices attached to the system
    let wmi = get_wmi()?;
    let query: Vec<HashMap<String, Variant>> = wmi
        .async_raw_query("SELECT * FROM Win32_DiskDrive")
        .await
        .map_err(|e| {
            let msg = format!("Failed to find disks through `wmi`. (err: {e})");
            tracing::warn!("{msg}");
            GhrError::ComponentInfoInaccessible(msg)
        })?;

    // for each device, we want to grab its info
    Ok(all(query).await)
}

/// finds info for all devices.
#[tracing::instrument]
pub(crate) async fn all(query: Vec<HashMap<String, Variant>>) -> Vec<ComponentInfo> {
    futures::stream::iter(query)
        .filter_map(one)
        .collect::<Vec<_>>()
        .await
}

/// finds info for one device.
#[tracing::instrument]
async fn one(device: HashMap<String, Variant>) -> Option<ComponentInfo> {
    // get model name and vendor
    let id = device.get("Model").and_then(|d| d.string_from_variant());
    let vendor_id = device
        .get("Manufacturer")
        .and_then(|m| m.string_from_variant());

    // let's find info about the disk size. speed appears to be unavailable.
    let size = device
        .get("Size")
        .and_then(|s| s.u64_from_variant())
        .map(|bytes| bytes / 1024);

    // and whether or not it's removable
    let is_removable = device
        .get("MediaType")
        .and_then(|mt| mt.string_from_variant())
        .map(|mt| matches!(mt.as_str(), "Removable media"));

    // build the description
    let desc = ComponentDescription::StorageDescription(StorageDescription {
        kind: None,
        usage: StorageUsage {
            usage: None,
            total_capacity: size,
        },
        speed: None,
        connector: None,
        is_removable,
    });

    Some(ComponentInfo {
        bus: ComponentBus::Unknown,
        id,
        class: None,
        vendor_id,
        status: None,
        desc,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn check_general_info() {
        logger();

        // get all info
        let query = sls2_battery_query().await;
        let all_info = all(query).await;
        let info = all_info.first().unwrap();

        // check name, vendor
        assert_eq!(info.id().unwrap(), "SAMSUNG MZ9LQ512HALU-00000");
        assert_eq!(info.vendor_id().unwrap(), "(Standard disk drives)"); // weird, but sure

        let ComponentDescription::StorageDescription(desc) = info.desc() else {
            panic!("wrong desc");
        };

        // check the capacity + that it isnt removable
        assert_eq!(desc.usage.total_capacity.unwrap(), 500103450);
        assert!(!desc.is_removable.unwrap())
    }

    #[tracing::instrument]
    async fn sls2_battery_query() -> Vec<HashMap<String, Variant>> {
        let root = env!("CARGO_MANIFEST_DIR");
        let path = PathBuf::from(root).join("tests/assets/windows/sls2_nvme_storage.json");
        serde_json::from_str(&tokio::fs::read_to_string(path).await.unwrap()).unwrap()
    }
}
