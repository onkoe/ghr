//! pci

use crate::prelude::internal::*;

#[tracing::instrument]
/// Gets a list of PCI devices on the system.
pub async fn get() -> GhrResult<Vec<ComponentInfo>> {
    // grab info about pci devices and construct reprs
    let mut pci = Vec::new();
    for dev in crate::report::components::linux::devices("/sys/bus/pci/devices").await? {
        // grab the component's path
        let path = dev.path();

        // load its vendor id and product name
        let (vendor, product, class) = tokio::join!(
            sysfs_value_opt::<String>(path.join("vendor")),
            sysfs_value_opt::<String>(path.join("device")),
            sysfs_value_opt::<String>(path.join("class")),
        );

        // read a few files to get important info about this thang
        let ((vendor_id, id), class) = (
            convert_to_pci_names(product, vendor),
            convert_to_pci_class(class),
        );

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

/// converts the device + vendor id to strings via pci lookup
#[tracing::instrument]
pub fn convert_to_pci_names(
    id: Option<String>,
    vendor_id: Option<String>,
) -> (Option<String>, Option<String>) {
    // make sure we've got both
    if let (Some(id), Some(vendor_id)) = (&id, &vendor_id) {
        // try converting them to hex
        if let (Ok(pid), Ok(vid)) = (
            u16::from_str_radix(&id.replace("0x", ""), 16),
            u16::from_str_radix(&vendor_id.replace("0x", ""), 16),
        ) {
            // try finding them a pci device
            if let Some(d) = pci_ids::Device::from_vid_pid(vid, pid) {
                // return a tuple of the id, vendor_id
                return (
                    Some(d.name().to_string()),
                    Some(d.vendor().name().to_string()),
                );
            }
        }
    }

    // otherwise, it didn't work. return the inputs
    tracing::warn!("Failed to get PCI identifiers.");
    (id, vendor_id)
}

#[tracing::instrument(skip(class))]
/// converts a given pci class identifier to a string.
///
/// if it's not able to do so, returns the given value.
pub fn convert_to_pci_class(class: Option<String>) -> Option<String> {
    // make sure we've got a class
    let Some((class, subclass)) = class.map(|c| {
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

    Some(class)
}
