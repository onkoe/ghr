use crate::prelude::internal::*;
use crate::report::components::windows::{get_pnp_with_did_prefix, get_wmi};

#[tracing::instrument]
pub async fn get() -> GhrResult<Vec<ComponentInfo>> {
    let wmi = get_wmi()?;
    get_pnp_with_did_prefix(wmi, "PCI").await
}
