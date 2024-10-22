use regex::Regex;
use tokio::fs::{self, DirEntry};

use crate::prelude::internal::*;

/// a static path where gpus reprs are placed by the kernel/drivers
const GPU_LISTING: &str = "/sys/class/drm";
/// gets the gpus on the system.
///
/// do not export this!
async fn devices() -> GhrResult<Vec<DirEntry>> {
    // to check if `drm/cardN` directories are the "main" ones
    let regex = Regex::new(r#"^card\d+$"#).map_err(|e| {
        tracing::error!("Regex creation failed!");
        GhrError::RegexCreationFailure(e.to_string())
    })?;

    // grab directory
    let mut entries = fs::read_dir(GPU_LISTING).await.map_err(|e| {
        tracing::error!(
            "Failed to read the `{GPU_LISTING}` directory on Linux, which should be static."
        );
        GhrError::ComponentInfoInaccessible(e.to_string())
    })?;

    // grab only the entries we want
    let mut gpus = Vec::new();
    while let Ok(Some(en)) = entries.next_entry().await {
        // we only want directories that look like `cardN`.
        let name = en.file_name();

        if regex.is_match(&name.to_string_lossy()) {
            gpus.push(en);
        }
    }

    Ok(gpus)
}
