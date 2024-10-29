use std::{
    fmt::{Debug, Display},
    path::Path,
    str::FromStr,
};

use crate::prelude::internal::*;

/// returns a value of type `V` from the file at `path`.
///
/// this string is trimmed to prevent parsing errors.
#[tracing::instrument]
pub(crate) async fn sysfs_value<V>(path: impl AsRef<Path> + Debug) -> GhrResult<V>
where
    V: FromStr,
    V::Err: Display, // ensure its error can be printed
{
    // read the file from disk
    let string = tokio::fs::read_to_string(&path).await.map_err(|e| {
        tracing::warn!("Failed to read string from `sysfs`.");
        GhrError::ComponentInfoInaccessible(format!(
            "Failed to read component info on `sysfs`. (path: `{path:?}`, err: {e}"
        ))
    })?;

    // attempt to parse the string into `V`
    string.trim().parse::<V>().map_err(|e| {
        tracing::error!("Value was expected to to parse into a `V`, but failed to do so. (value: `{string}`, err: {e})");
        GhrError::ComponentInfoWeirdInfo(format!("Failed to parse value from string. (value: `{string}`, err: {e})"))
    })
}

/// returns a value of type `V` from the file at `path`.
///
/// this string is trimmed to prevent parsing errors.
///
/// note that this version of the function ignores any errors and converts
/// directly to `Option` instead.
#[tracing::instrument]
pub(crate) async fn sysfs_value_opt<V>(path: impl AsRef<Path> + Debug) -> Option<V>
where
    V: FromStr,
    V::Err: Display, // ensure its error can be printed
{
    sysfs_value::<V>(path).await.ok()
}

/// class, id, vendor
///
/// (assumes that the given device is pci or uses pci values)
pub(crate) struct Civ {
    pub(crate) class: Option<String>,
    pub(crate) vendor: Option<String>,
    pub(crate) id: Option<String>,
}

impl Civ {
    /// grabs the `sysfs` device's pci class/vendor_id/id.
    ///
    /// this path MUST be a `sysfs` entry, like `/sys/class/<class>/<device>`.
    /// do not use other paths.
    #[tracing::instrument]
    pub(crate) async fn new<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Self {
        let path = path.as_ref().join("device/");

        // read all three values
        let (class, id, vendor) = tokio::join! {
            sysfs_value_opt::<String>(path.join("class")),
            sysfs_value_opt::<String>(path.join("device")),
            sysfs_value_opt::<String>(path.join("vendor")),
        };

        let (class, (id, vendor)) = (
            convert_to_pci_class(class),
            convert_to_pci_names(id, vendor),
        );

        Self { class, vendor, id }
    }
}

