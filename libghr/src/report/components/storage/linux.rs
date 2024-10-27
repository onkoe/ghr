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
    // (get info)

    todo!()
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
    async fn todo_test() {
        logger();
        todo!()
    }
}
