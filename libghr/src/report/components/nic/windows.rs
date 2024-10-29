use crate::prelude::internal::*;

/// Finds and returns info about network devices on the system.
#[tracing::instrument]
pub(crate) async fn get() -> GhrResult<Vec<ComponentInfo>> {
    tracing::error!("Network interface detection isn't yet implemented on Windows.");
    Ok(vec![])
}
