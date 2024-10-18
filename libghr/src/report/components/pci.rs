//! pci

use crate::prelude::internal::*;

use tokio::try_join;

#[cfg(target_os = "linux")]
pub async fn pci_components() -> GhrResult<Vec<ComponentInfo>> {
    // grab info about pci devices and construct reprs
    let mut pci = Vec::new();
    for dev in super::linux::devices("/sys/bus/pci/devices").await? {
        // grab the component's path
        let path = dev.path();

        // read a few files to get important info about this thang
        let Ok((id, vendor_id, class)) = try_join!(
            tokio::fs::read_to_string(path.join("device")),
            tokio::fs::read_to_string(path.join("vendor")),
            tokio::fs::read_to_string(path.join("class")),
        ) else {
            tracing::warn!(
                "A USB component's info was inaccessible! (device at `{})`",
                path.display()
            );
            continue;
        };

        pci.push(ComponentInfo {
            bus: ComponentBus::Pci,
            id,
            class,
            vendor_id,
            status: ComponentStatus {}, // TODO
            desc: ComponentDescription::None,
        })
    }

    Ok(pci)
}
