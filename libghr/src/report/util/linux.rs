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
