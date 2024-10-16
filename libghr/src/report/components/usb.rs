//! i am usb

use super::{ComponentBus, ComponentInfo, ComponentStatus};
use crate::prelude::internal::*;

use tokio::try_join;

#[cfg(target_os = "linux")]
pub async fn usb_components() -> GhrResult<Vec<ComponentInfo>> {
    // grab info about usb devices and construct reprs
    let mut usb = Vec::new();
    for dev in super::linux::devices("/sys/bus/usb/devices").await? {
        // grab the component's path
        let path = dev.path();

        // read a few files to get important info about this thang
        let Ok((id, vendor_id, class)) = try_join!(
            tokio::fs::read_to_string(path.join("idProduct")),
            tokio::fs::read_to_string(path.join("idVendor")),
            tokio::fs::read_to_string(path.join("bInterfaceClass")),
        ) else {
            tracing::warn!(
                "A USB component's info was inaccessible! (device at `{})`",
                path.display()
            );
            continue;
        };

        usb.push(ComponentInfo {
            bus: ComponentBus::Usb,
            id,
            class,
            vendor_id,
            status: ComponentStatus {}, // TODO
        })
    }

    Ok(usb)
}
