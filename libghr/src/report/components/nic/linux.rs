use std::path::{Path, PathBuf};

use futures::StreamExt;
use tokio_stream::wrappers::ReadDirStream;

use crate::prelude::internal::*;

/// find and returns info about network devices on the system.
#[tracing::instrument]
pub(crate) async fn get() -> GhrResult<Vec<ComponentInfo>> {
    let net_sysfs = PathBuf::from("/sys/class/net");

    // run for the sysfs
    all(net_sysfs).await
}

/// gets info about all devices at the given path.
#[tracing::instrument]
async fn all<P: AsRef<Path> + std::fmt::Debug>(path: P) -> GhrResult<Vec<ComponentInfo>> {
    let entries = tokio::fs::read_dir(path).await.map_err(|e| {
        GhrError::ComponentInfoInaccessible(format!(
            "Couldn't read network interface devices from `sysfs`. (err: {e})"
        ))
    })?;

    // iterate over entries, only use good paths, then run them through `one`
    Ok(ReadDirStream::new(entries)
        .map(|res| res.map(|entry| entry.path()))
        .filter_map(|res| async { res.ok() })
        .filter_map(one)
        .collect()
        .await)
}

/// fetches info about the device at the given path.
///
/// path is to a `sysfs` networking device representation, generally
/// at `/sys/class/net/<device>`.
#[tracing::instrument]
async fn one<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Option<ComponentInfo> {
    let path = path.as_ref();

    // grab pci class, name, and vendor
    let (class, name, vendor) = tokio::join! {
        sysfs_value_opt::<String>(path.join("device/class")),
        sysfs_value_opt::<String>(path.join("device/device")),
        sysfs_value_opt::<String>(path.join("device/vendor")),
    };

    // map them to the fr ones
    let (class, (name, vendor)) = (
        convert_to_pci_class(class),
        convert_to_pci_names(name, vendor),
    );

    // also grab the speed and mtu from `sysfs`
    let (speed, mtu) = tokio::join! {
        sysfs_value_opt::<u32>(path.join("speed")),
        sysfs_value_opt::<u32>(path.join("mtu")),
    };

    Some(ComponentInfo {
        bus: ComponentBus::Unknown,
        id: name,
        class,
        vendor_id: vendor,
        status: None,
        desc: ComponentDescription::NicDescription(NicDescription {
            max_speed: speed,
            mtu,
        }),
    })
}
