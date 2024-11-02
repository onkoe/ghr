use std::collections::HashMap;

use futures::StreamExt;
use wmi::Variant;

use crate::prelude::internal::*;

#[tracing::instrument]
pub(crate) async fn get() -> GhrResult<Vec<ComponentInfo>> {
    let wmi = get_wmi()?;

    // query wmi for network interfaces
    let query: Vec<HashMap<String, Variant>> = wmi
        .async_raw_query("SELECT * from Win32_Battery")
        .await
        .map_err(|e| {
            tracing::warn!("Couldn't get battery info from `wmi`. (err: {e})");
            GhrError::ComponentInfoInaccessible(format!(
                "Couldn't get battery info from `wmi`. (err: {e})"
            ))
        })?;

    // send the query for parsing
    Ok(all(query).await)
}

#[tracing::instrument]
async fn all(query: Vec<HashMap<String, Variant>>) -> Vec<ComponentInfo> {
    futures::stream::iter(query).then(one).collect().await
}

#[tracing::instrument]
async fn one(fields: HashMap<String, Variant>) -> ComponentInfo {
    // get name and vendor
    let name = fields.get("Name").and_then(|v| v.string_from_variant());
    let vendor = None;

    // grab capacity (ideal and real)
    let real_capacity_wh = fields
        .get("FullChargeCapacity")
        .u32_from_variant()
        .map(|mwh| mwh as f64 / 1000_f64);
    let theoretical_capacity_wh = fields
        .get("DesignCapacity")
        .u32_from_variant()
        .map(|mwh| mwh as f64 / 1000_f64);

    // get technology
    let technology = fields
        .get("Chemistry")
        .inspect(|c| tracing::debug!("battery chemistry id: `{c:#?}`"))
        .u32_from_variant()
        .and_then(|num| match num {
            1 | 2 => None,
            3 => Some("Lead Acid".into()),
            4 => Some("Nickel Cadmium".into()),
            5 => Some("Nickel Metal Hydride".into()),
            6 => Some("Lithium-ion".into()),
            7 => Some("Zinc air".into()),
            8 => Some("Lithium Polymer".into()),
            other => {
                tracing::warn!("Got a weird battery chemistry (number: `{other}`).");
                None
            }
        });

    // make the device
    ComponentInfo {
        bus: ComponentBus::Unknown,
        id: name,
        class: None,
        vendor_id: vendor,
        status: None,
        desc: ComponentDescription::PowerSupplyDescription(PowerSupplyDescription::Battery {
            technology,
            real_capacity_wh,
            theoretical_capacity_wh,
            cycle_count: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn check_sls2_battery() {
        let path = sls2_battery_path();
        let query = serde_json::from_str(&tokio::fs::read_to_string(path).await.unwrap()).unwrap();

        let batteries = all(query).await;
        let cmp = batteries.first().unwrap();

        // name and vendor
        assert_eq!(cmp.id().unwrap(), "SurfaceBattery");
        assert!(cmp.vendor_id().is_none());

        // we should have a matching description
        let ComponentDescription::PowerSupplyDescription(PowerSupplyDescription::Battery {
            technology,
            real_capacity_wh,
            theoretical_capacity_wh,
            cycle_count,
        }) = cmp.desc()
        else {
            panic!("wrong one!");
        };

        // check desc attributes
        assert_eq!(technology.unwrap(), "Lithium-ion");
        assert!(almost::equal(real_capacity_wh.unwrap(), 49.5));
        assert!(almost::equal(theoretical_capacity_wh.unwrap(), 50.0));
        assert!(cycle_count.is_none()); // windows doesn't seem to support this
    }

    #[tracing::instrument]
    fn sls2_battery_path() -> PathBuf {
        let root = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(root).join("tests/assets/windows/sls2_battery.json")
    }
}
