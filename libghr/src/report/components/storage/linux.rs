use futures::StreamExt as _;
use tokio_stream::wrappers::ReadDirStream;

use crate::prelude::internal::*;
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

/// grabs all known linux `block` devices from `sysfs`.
#[tracing::instrument]
pub(crate) async fn get() -> GhrResult<Vec<ComponentInfo>> {
    // grab storage devices attached to the system
    let devices = storage_device_entries("/sys/class/block").await?;

    // for each device, we want to grab its info
    Ok(futures::stream::iter(devices)
        .filter_map(one)
        .collect::<Vec<_>>()
        .await)
}

/// finds info for one device.
#[tracing::instrument]
async fn one<P: AsRef<Path> + Debug>(path: P) -> Option<ComponentInfo> {
    // grab general info
    let (id, vendor_id) = general_info(&path).await;

    // and the specialized desc
    let desc = ComponentDescription::StorageDescription(storage_desc(path).await);

    Some(ComponentInfo {
        bus: ComponentBus::Unknown, // FIXME: convert the desc's if we got it
        id,
        class: None,
        vendor_id,
        status: None,
        desc,
    })
}

/// finds general info for the given device.
///
/// currently, the return type is: (id, vendor_id)
#[tracing::instrument]
async fn general_info<P: AsRef<Path> + Debug>(path: P) -> (Option<String>, Option<String>) {
    // find the real path
    let path = path.as_ref();

    tokio::join! {
        sysfs_value_opt(path.join("device/model")),
        sysfs_value_opt(path.join("device/vendor"))
    }
}

/// finds the specialized device description for the device at `path`.
#[tracing::instrument]
async fn storage_desc<P: AsRef<Path> + Debug>(path: P) -> StorageDescription {
    // grab the real path
    let path = path.as_ref();

    let (kind, capacity, speed, connector, is_removable) = tokio::join! {
        kind(path),
        capacity(path),
        sysfs_value_opt::<u32>(path.join("queue/rotation_rate")), // rare?
        connector(path),
        removable(path),
    };

    // FIXME: `rotation_rate` isn't well-documented online.
    //        make sure it exists and isn't unique to like
    //        four drivers...

    let usage = StorageUsage {
        usage: None,
        total_capacity: capacity,
    };

    StorageDescription {
        kind,
        usage,
        speed,
        connector,
        is_removable,
    }
}

/// finds the connector for the device at `path`.
#[tracing::instrument]
async fn connector<P: AsRef<Path> + Debug>(path: P) -> Option<StorageConnector> {
    // grab the real path
    let path = path.as_ref();

    // follow any symlinks
    let abs = path.canonicalize().unwrap_or(path.to_path_buf());

    // we now have a `<sysroot>/sys/class/<connector>` path.
    //
    // let's try to match a `/class/<connector>` from its components.
    let mut chunks = futures::stream::iter(abs.components()).chunks(2);
    while let Some(chunk) = chunks.next().await {
        let first = chunk.first();

        if first.is_none() {
            break;
        }

        if let (Some(class), Some(connector)) = (first, chunk.get(1)) {
            // make sure it's a fr path
            if class.as_os_str().to_string_lossy() == "class" {
                // try matching off a connector
                if let Some(connector) = connector.as_os_str().to_str() {
                    match connector {
                        "ata_port" => return Some(StorageConnector::Sata),
                        "scsi_disk" => return Some(StorageConnector::Scsi),
                        other => {
                            tracing::debug!("Failed to map storage connector `{other}`.");
                        } //
                          //
                          // TODO: consider pci? i didn't see it in `/sys/class` though
                    };
                }
            }
        }
    }

    None
}

/// sees if the device at `path` is removable.
#[tracing::instrument]
async fn removable<P: AsRef<Path> + Debug>(path: P) -> Option<bool> {
    // grab the real path
    let path = path.as_ref();

    // read the `removable` file from `sysfs`
    let removable_switch = sysfs_value_opt::<u8>(path.join("removable")).await?;

    // map the number `1` to yes, `0` to no... otherwise no clue
    match removable_switch {
        1_u8 => Some(true),
        0_u8 => Some(false),

        other => {
            tracing::debug!("uhh. this drive was reported to have a removable value of `{other}`!");
            None
        }
    }
}

/// sees if the device at `path` is rotational.
#[tracing::instrument]
async fn kind<P: AsRef<Path> + Debug>(path: P) -> Option<StorageKind> {
    // grab the real path
    let path = path.as_ref();

    // read file from the `sysfs`
    let rotational_switch = sysfs_value_opt::<u8>(path.join("queue/rotational")).await?;

    // map number to bool
    match rotational_switch {
        1_u8 => Some(StorageKind::Hdd),
        0_u8 => Some(StorageKind::Ssd),

        other => {
            tracing::debug!("uhh... a drive has a rotational value of `{other}`!");
            None
        }
    }
}

/// attempts to find a device's capacity in KiB.
#[tracing::instrument]
async fn capacity<P: AsRef<Path> + Debug>(path: P) -> Option<u64> {
    let path = path.as_ref();

    // read the value...
    //
    // note: it's in sectors!
    let (sector_count, sector_width) = tokio::join! {
        sysfs_value_opt::<u64>(path.join("size")),
        sysfs_value_opt::<u64>(path.join("queue/physical_block_size"))
    };

    // make sure the values are all good
    let (Some(sector_count), Some(sector_width)) = (sector_count, sector_width) else {
        tracing::warn!("Failed to find drive capacity. (sectors: `{sector_count:?}`, width: `{sector_width:?}`)");
        return None;
    };

    // B = (sector_size * sector_count)
    // ...KiB = (B / 1024)
    Some((sector_width * sector_count) / 1024)
}

/// finds all (non-partition) entries in the given `<sysfs>/class/block` path.
#[tracing::instrument]
async fn storage_device_entries<P: AsRef<Path> + Debug>(path: P) -> GhrResult<Vec<PathBuf>> {
    // grab all entries from that dir
    let entries = tokio::fs::read_dir(path).await.map_err(|e| {
        GhrError::ComponentInfoInaccessible(format!(
            "Failed to read block devices from `sysfs` (err: {e})"
        ))
    })?;

    // use futures::stream to grab each entry's path
    let paths = ReadDirStream::new(entries)
        .map(|res| res.map(|entry| entry.path()))
        .filter_map(|res| async { res.ok() })
        .collect::<Vec<_>>()
        .await;

    // now, we'll grab each path's last component (dir name)
    let names = futures::stream::iter(paths.clone())
        .filter_map(|path| async move {
            path.file_name()
                .and_then(|n| n.to_str().map(|s| s.to_string()))
        })
        .collect::<Vec<_>>()
        .await;

    // finally, check for devices (`sda`) and remove any partitions (`sda1`)
    let devices = futures::stream::iter(paths)
        .filter_map(|path| async {
            let fr_path = tokio::fs::canonicalize(&path).await.ok()?;
            tracing::error!("REMOVE ME(FOUND ENTRY AT PATH): {fr_path:#?}");

            // check if the path's second-to-last path component contains any of
            // the dir names we got.
            //
            // if so, it's a partition of a device and won't give useful info!
            let is_device = !names.iter().any(|name| {
                fr_path
                    .components()
                    .rev()
                    .nth(1)
                    .map_or(true, |second_to_last| {
                        second_to_last.as_os_str().to_string_lossy() == *name
                    })
            });

            // only keep the path in if it's a device
            if is_device {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .await;

    Ok(devices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn check_general_info() {
        logger();
        let path = ssd_path();

        // get all info
        let info = one(path).await.unwrap();

        // check name, vendor
        assert_eq!(info.id.unwrap(), "Samsung SSD 860");
        assert_eq!(info.vendor_id.unwrap(), "ATA");
    }

    #[tokio::test]
    async fn check_specialized_info() {
        logger();
        let path = ssd_path();

        // get all info
        let info = one(path).await.unwrap();

        // ensure we've got a drive
        let ComponentDescription::StorageDescription(desc) = info.desc() else {
            panic!("wrong desc ty!");
        };

        // do checks
        //
        // removed til' i can simulate symlinks
        // assert_eq!(desc.connector.unwrap(), StorageConnector::Sata);
        //
        assert_eq!(desc.is_removable, Some(false));
        assert_eq!(desc.kind.unwrap(), StorageKind::Ssd);
        assert!(desc.speed.is_none());
        assert_eq!(
            desc.usage.total_capacity.unwrap(),
            (512 * 1953525168) / 1024
        );
    }

    #[tokio::test]
    async fn check_connector() {
        logger();

        let _a = connector("/home/farts/sys/class/scsi_disk/15:0:0:0/")
            .await
            .unwrap();
        let _b = connector("/sys/class/ata_port/1:0:0:0/").await.unwrap();
        let _c = connector("/sys/class/scsi_disk/any_name_will_do")
            .await
            .unwrap();
        let _d = connector("/home/sys/class/real_sysroot/sys/class/scsi_disk/15:0:0:0/")
            .await
            .unwrap();

        assert_eq!(_a, StorageConnector::Scsi);
        assert_eq!(_b, StorageConnector::Sata);
        assert_eq!(_c, StorageConnector::Scsi);
        assert_eq!(_d, StorageConnector::Scsi);
    }

    #[tokio::test]
    async fn check_capacity() {
        logger();
        let path = ssd_path();

        // ensure it's converting to KiB correctly
        let cap = capacity(path).await.unwrap();
        assert_eq!(cap, ((512 * 1953525168) / 1024));

        // check in GiB, too
        let cap_gib = cap as f64 / 1024_f64.powf(2.0);
        assert!(almost::equal(cap_gib, 931.51339))
    }

    #[tokio::test]
    async fn check_speed() {
        logger();
        let ComponentDescription::StorageDescription(desc) = one(hdd_path()).await.unwrap().desc()
        else {
            panic!("wrong desc ty!");
        };

        // speed is set to 7200 rpm.
        assert_eq!(desc.speed.unwrap(), 7200);
    }

    #[tracing::instrument]
    fn ssd_path() -> PathBuf {
        let root = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(format!(
            "{root}/tests/assets/linux/sysfs/sys/class/block/sda"
        ))
    }

    fn hdd_path() -> PathBuf {
        let root = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(format!(
            "{root}/tests/assets/linux/sysfs/sys/class/block/sdb"
        ))
    }
}
