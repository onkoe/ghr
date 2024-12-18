use std::path::{Path, PathBuf};

use crate::{prelude::internal::*, report::components};

/// Grabs info about all system power supplies.
#[tracing::instrument]
pub async fn get() -> GhrResult<Vec<ComponentInfo>> {
    // the path to all linux power supplies is located at `/sys/class/power_supply`.
    let power_supply_path = PathBuf::from("/sys/class/power_supply");

    let mut psus = Vec::new();
    for dev in components::linux::devices(power_supply_path).await? {
        // grab the component's path
        let path = dev.path();

        if let Some(unit) = one(path).await {
            psus.push(unit)
        }
    }

    Ok(psus)
}

/// returns a representation of a component at `path`, if one exists.
#[tracing::instrument]
async fn one<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Option<ComponentInfo> {
    // get a reference to the given path
    let path = path.as_ref();

    // read a few files to get important info about this thang
    let (vendor_id, id, kind) = futures::join! {
        sysfs_value_opt::<String>(path.join("manufacturer")),
        sysfs_value_opt::<String>(path.join("model_name")),
        sysfs_value_opt::<String>(path.join("type")),
    };

    // get extra info depending on the type of supply
    let psu_info = if let Some(kind) = kind {
        psu_info(&kind, path)
            .await
            .map(ComponentDescription::PowerSupplyDescription)
            .unwrap_or(ComponentDescription::None)
    } else {
        ComponentDescription::None
    };

    Some(ComponentInfo {
        bus: ComponentBus::Sys,
        id,
        class: None,
        vendor_id,
        status: None,
        desc: psu_info,
    })
}

#[tracing::instrument]
/// matches on the type of power supply this device is and runs the appropriate
/// function
async fn psu_info(kind: &str, path: &Path) -> Option<PowerSupplyDescription> {
    match kind {
        "Battery" => battery_info(path).await,
        _ => None,
    }
}

#[tracing::instrument]
/// finds info about a battery
async fn battery_info(path: &Path) -> Option<PowerSupplyDescription> {
    // this is a lot of stuff but it's fine
    let (
        cycle_count,
        technology,
        energy_full_design_uwh,
        energy_full_uwh,
        charge_full_design_uah,
        charge_full_uah,
        voltage_max_uv,
        voltage_max_design_uv,
    ) = futures::join! {
        sysfs_value_opt::<i32>(path.join("cycle_count")),
        sysfs_value_opt::<String>(path.join("technology")),
        sysfs_value_opt::<u64>(path.join("energy_full_design")),
        sysfs_value_opt::<u64>(path.join("energy_full")),
        sysfs_value_opt::<u64>(path.join("charge_full")),
        sysfs_value_opt::<u64>(path.join("charge_full_design")),
        sysfs_value_opt::<u64>(path.join("voltage_max")),
        sysfs_value_opt::<u64>(path.join("voltage_max_design")),
    };

    // change technology: `Some("Unknown")` to `None`
    let technology = technology.and_then(|t| if t == "Unknown" { None } else { Some(t) });

    // this lambda calculates the 'actual' battery capacity in wh.
    //
    // we'd prefer reading the `energy` value directly, but when it's not
    // present, we'll estimate based on the uv + uah values
    let calc_cap = |energy_uwh, charge_uah, voltage_uv| {
        if let Some(known_uwh) = energy_uwh {
            Some(uwh_to_wh(known_uwh))
        } else if let (Some(known_charge_uah), Some(known_voltage_uv)) = (charge_uah, voltage_uv) {
            Some(uwh_to_wh(known_charge_uah * known_voltage_uv))
        } else {
            None
        }
    };

    // let's calculate the battery's capacity in watt-hours
    let real_capacity_wh = calc_cap(energy_full_uwh, charge_full_uah, voltage_max_uv);
    let theoretical_capacity_wh = calc_cap(
        energy_full_design_uwh,
        charge_full_design_uah,
        voltage_max_design_uv,
    );

    Some(PowerSupplyDescription::Battery {
        technology,
        real_capacity_wh,
        theoretical_capacity_wh,
        cycle_count,
    })
}

#[tracing::instrument]
/// converts microwatt-hours to watt-hours
fn uwh_to_wh(uwh: u64) -> f64 {
    (uwh as f64) / 1_000_000_f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn psu_linux_info() {
        let path = psu_linux_path().join("BAT1");
        let psu = one(path).await.unwrap();

        // check vendor
        assert_eq!(psu.vendor_id.unwrap(), "SMP");

        // check model (yes, this sample was a number)
        assert_eq!(psu.id.unwrap(), "1144021016");

        // i haven't yet seen a class on linux. so we'll test on it
        assert!(psu.class.is_none());
    }

    #[tokio::test]
    async fn psu_linux_specs() {
        let path = psu_linux_path().join("BAT1");
        let psu = one(path).await.unwrap();

        let ComponentDescription::PowerSupplyDescription(psu_info) = psu.desc else {
            panic!("no psu info found D:");
        };

        let PowerSupplyDescription::Battery {
            technology,
            real_capacity_wh,
            theoretical_capacity_wh,
            cycle_count,
        } = psu_info
        else {
            panic!("wasn't considered a battery");
        };

        // battery composition (technology)
        assert_eq!(technology.unwrap(), "Li-ion");

        // ideal capacity
        assert!(almost::equal(theoretical_capacity_wh.unwrap(), 56.31));

        // fr capacity
        assert!(almost::equal(real_capacity_wh.unwrap(), 52.22));

        // cycle count
        assert_eq!(cycle_count.unwrap(), 37);
    }

    #[test]
    fn check_uwh_conversion() {
        let wh = 99.0;
        let uwh = 99_000_000;

        assert!(almost::equal(uwh_to_wh(uwh), wh));
    }

    #[tracing::instrument]
    fn psu_linux_path() -> PathBuf {
        let root = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(format!(
            "{root}/tests/assets/linux/sysfs/sys/class/power_supply"
        ))
    }
}
