use crate::prelude::internal::*;

#[tracing::instrument]
pub async fn get() -> GhrResult<Vec<ComponentInfo>> {
    tracing::error!("Power supply info is unimplemented on Windows.");
    Ok(Vec::new()) // TODO
}
