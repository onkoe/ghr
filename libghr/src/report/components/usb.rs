//! i am usb

use crate::prelude::internal::*;

#[cfg(target_os = "linux")]
use std::path::Path;
#[cfg(target_os = "linux")]
use tokio::join;
#[cfg(target_os = "linux")]
use tokio::try_join;

#[tracing::instrument]
#[cfg(target_os = "linux")]
pub async fn get() -> GhrResult<Vec<ComponentInfo>> {
    // grab info about usb devices and construct reprs
    let mut usb = Vec::new();
    for dev in super::linux::devices("/sys/bus/usb/devices").await? {
        // grab the component's path
        let path = dev.path();

        // read a few files to get important info about this thang
        let ((vendor_id, id), class) = join! {
            usb_vendor_and_id(&path),
            usb_class(&path)
        };

        usb.push(ComponentInfo {
            bus: ComponentBus::Usb,
            id,
            class,
            vendor_id,
            status: None,
            desc: ComponentDescription::None,
        })
    }

    Ok(usb)
}

/// on Linux, this grabs the class codes for our devices.
#[cfg(target_os = "linux")]
#[tracing::instrument]
async fn usb_class(path: &Path) -> Option<String> {
    // read the files
    let Ok((class, subclass)) = try_join!(
        tokio::fs::read_to_string(path.join("bDeviceClass")),
        tokio::fs::read_to_string(path.join("bDeviceSubClass")),
    ) else {
        tracing::warn!(
            "Failed to find device class/subclass! Yielding 'Unknown' for these values."
        );
        return None;
    };

    let (class, subclass) = (class.trim(), subclass.trim());

    // try grabbing a class
    // look it up
    if let (Ok(parsed_class), Ok(parsed_subclass)) = (
        u8::from_str_radix(class, 16),
        u8::from_str_radix(subclass, 16),
    ) {
        if let Some(c) = usb_ids::SubClass::from_cid_scid(parsed_class, parsed_subclass) {
            return Some(format!("{} ({})", c.class().name(), c.name()));
        }
    }

    // return the raw ID if we must
    return Some(format!("{} ({})", class, subclass));
}

#[cfg(target_os = "linux")]
#[tracing::instrument]
async fn usb_vendor_and_id(path: &Path) -> (Option<String>, Option<String>) {
    // look for human-readable string repr

    use usb_ids::FromId as _;
    let human_readable = tokio::join! {
        tokio::fs::read_to_string(path.join("iManufacturer")),
        tokio::fs::read_to_string(path.join("iProduct")),
    };

    // return human-readable strings if they exist for us!
    if let (Ok(vendor), Ok(product)) = human_readable {
        tracing::debug!(
            "oh cool, we got human readable strings! vendor: `{vendor}`. product: {product}."
        );
        return (Some(vendor), Some(product));
    }

    // otherwise, grab lame number
    let (vend, prod) = tokio::join! {
        tokio::fs::read_to_string(path.join("idVendor")),
        tokio::fs::read_to_string(path.join("idProduct")),
    };

    // trim whitespaces (newlines)
    let (mut vend, prod) = (
        vend.map(|s| s.trim().to_string()),
        prod.map(|s| s.trim().to_string()),
    );

    // convert them if they exist.
    if let (Ok(ref num_vend), Ok(ref num_prod)) = (&vend, &prod) {
        tracing::debug!("lame numbers {}; {}", num_vend, num_prod);

        // try to convert them to human readable names
        let (parsed_vend, parsed_prod) = (
            u16::from_str_radix(num_vend, 16),
            u16::from_str_radix(num_prod, 16),
        );

        if let (Ok(parsed_vend), Ok(parsed_prod)) = (parsed_vend, parsed_prod) {
            tracing::debug!("parsed from str radix");
            if let Some(dev) = usb_ids::Device::from_vid_pid(parsed_vend, parsed_prod) {
                tracing::debug!("names");
                return (Some(dev.vendor().name().into()), Some(dev.name().into()));
            }

            // sometimes, the linux usb list is incomplete. let's try just the vendor!
            if let Some(readable_vend) = usb_ids::Vendor::from_id(parsed_vend) {
                vend = Ok(readable_vend.name().to_string());
            }
        }
    }

    // otherwise, report errors and return "Unknown" strings
    if let Err(prod_err) = &prod {
        tracing::warn!(
            "Couldn't find product for USB device at path {path:?}. (error: {prod_err})"
        );
    }
    if let Err(vend_err) = &vend {
        tracing::warn!("Couldn't find vendor for USB device at path {path:?}. (error: {vend_err})");
    }

    // give up
    (None, None)
}

#[tracing::instrument]
#[cfg(target_os = "windows")]
pub async fn get() -> GhrResult<Vec<ComponentInfo>> {
    use super::windows::get_pnp_with_did_prefix;
    use crate::report::components::windows::get_wmi;

    let wmi = get_wmi()?;
    get_pnp_with_did_prefix(wmi, "USB").await
}
