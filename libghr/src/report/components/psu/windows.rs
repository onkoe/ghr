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

async fn all(query: Vec<HashMap<String, Variant>>) -> Vec<ComponentInfo> {
    futures::stream::iter(query).then(one).collect().await
}

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
}
