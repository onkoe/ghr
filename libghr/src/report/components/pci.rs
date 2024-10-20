//! pci

use crate::prelude::internal::*;

#[cfg(target_os = "linux")]
use std::path::Path;
#[cfg(target_os = "linux")]
use tokio::join;

#[cfg(target_os = "linux")]
pub async fn pci_components() -> GhrResult<Vec<ComponentInfo>> {
    // grab info about pci devices and construct reprs
    let mut pci = Vec::new();
    for dev in super::linux::devices("/sys/bus/pci/devices").await? {
        // grab the component's path
        let path = dev.path();

        // read a few files to get important info about this thang
        let ((vendor_id, id), class) = join! {
            pci_vendor_id(&path),
            pci_class(&path),
        };

        pci.push(ComponentInfo {
            bus: ComponentBus::Pci,
            id,
            class,
            vendor_id,
            status: None, // TODO
            desc: ComponentDescription::None,
        })
    }

    Ok(pci)
}

/// grabs the PCI class on Linux.
#[tracing::instrument]
#[cfg(target_os = "linux")]
async fn pci_class(path: &Path) -> Option<String> {
    // we'll load its class
    let class = tokio::fs::read_to_string(path.join("class")).await;

    // make sure we've got a class
    let Ok((class, subclass)) = class.map(|c| {
        let string = c.trim().to_string().replace("0x", "");
        let split = string.split_at(2);

        let (c, mut subc) = (split.0.to_string(), split.1.to_string());
        subc.truncate(2);
        (c, subc)
    }) else {
        tracing::warn!("Failed to find the class of this PCI device.");
        return None;
    };

    // make sure we can convert it into a number
    let (Ok(class_num), Ok(subclass_num)) = (
        u8::from_str_radix(&class, 16),
        u8::from_str_radix(&subclass, 16),
    ) else {
        tracing::warn!("Class was not a number: {}", &class);
        return None;
    };

    // we do! let's parse it for an ID we understand
    if let Some(parsed) = pci_ids::Subclass::from_cid_sid(class_num, subclass_num) {
        return Some(format!("{} ({})", parsed.class().name(), parsed.name()));
    }

    return Some(class);
}

/// grabs the PCI vendor and product names on Linux.
#[tracing::instrument]
#[cfg(target_os = "linux")]
async fn pci_vendor_id(path: &Path) -> (Option<String>, Option<String>) {
    // load its vendor id and product name
    let (Ok(vendor), Ok(product)) = tokio::join!(
        tokio::fs::read_to_string(path.join("vendor")),
        tokio::fs::read_to_string(path.join("device")),
    ) else {
        tracing::warn!("Failed to get vendor info for this PCI device.");
        return (None, None);
    };

    // trim the strings we got
    let (vendor, product) = (vendor.trim().to_string(), product.trim().to_string());

    // parse these into numbers
    let (Ok(num_vendor), Ok(num_product)) = (
        u16::from_str_radix(&vendor.replace("0x", ""), 16),
        u16::from_str_radix(&product.replace("0x", ""), 16),
    ) else {
        tracing::warn!("PCI vendor info was not parseable as a number.");
        return (Some(vendor), Some(product));
    };

    // use them
    if let Some(device) = pci_ids::Device::from_vid_pid(num_vendor, num_product) {
        return (
            Some(device.vendor().name().to_string()),
            Some(device.name().to_string()),
        );
    }

    // the listing didn't have these devices present!
    // we'll just return the raw numeric values instead.
    (Some(vendor), Some(product))
}

#[cfg(target_os = "windows")]
#[tracing::instrument]
pub async fn pci_components() -> GhrResult<Vec<ComponentInfo>> {
    use super::windows::get_pnp_with_did_prefix;
    use crate::report::components::windows::get_wmi;

    let wmi = get_wmi()?;
    get_pnp_with_did_prefix(wmi, "PCI").await
}

/*
#[cfg(target_os = "windows")]
#[tracing::instrument]
pub async fn pci_components() -> GhrResult<Vec<ComponentInfo>> {
    use std::collections::HashMap;
    use windows::Win32::System::Wmi::CIMTYPE_ENUMERATION;
    use wmi::{FilterValue, Variant};

    use crate::report::components::windows::{get_pnp, get_wmi};
    let wmi_connection = get_wmi()?;

    // grab list of all pnp devices on windows
    let pnp = get_pnp(wmi_connection).await?;
    dbg!(pnp.len());

    // filter devices for the pci ones
    tracing::debug!("filtering pci devices...");
    Ok(pnp
        .into_iter()
        .map(|pnp_device| {
            (
                pnp_device
                    .get("DeviceID")
                    .and_then(|n| {
                        if let Variant::String(s) = n {
                            Some(s)
                        } else {
                            None
                        }
                    })
                    .cloned(),
                pnp_device,
            )
        })
        .filter(|(device_id, _pnp_device)| {
            device_id
                .clone()
                .filter(|did| did.trim().to_uppercase().starts_with("PCI"))
                .is_some()
        })
        .map(|(_, pnp_device)| pnp_device)
        .map(|pci_device| {
            // asdf
            let id = pci_device
                .get("Name")
                .and_then(|n| {
                    if let Variant::String(s) = n {
                        Some(s)
                    } else {
                        None
                    }
                })
                .cloned();

            let class = pci_device
                .get("PNPClass")
                .and_then(|n| {
                    if let Variant::String(s) = n {
                        Some(s)
                    } else {
                        None
                    }
                })
                .cloned();

            let vendor_id = pci_device
                .get("Manufacturer")
                .and_then(|n| {
                    if let Variant::String(s) = n {
                        Some(s)
                    } else {
                        None
                    }
                })
                .cloned();

            ComponentInfo {
                bus: ComponentBus::Pci,
                id,
                class,
                vendor_id,
                status: None,
                desc: ComponentDescription::None,
            }
        })
        .collect::<Vec<ComponentInfo>>())
}
*/
