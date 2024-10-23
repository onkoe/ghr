use std::path::Path;

use crate::prelude::internal::*;

// TODO: find out what all gpus on linux should provide.
//
// (see `asahi-drm` for a nice, readable impl)

#[tracing::instrument]
pub(super) async fn gpu(_path: &Path) -> GhrResult<ComponentInfo> {
    // create a useless
    Ok(ComponentInfo {
        bus: ComponentBus::Pci,
        id: None,
        class: None,
        vendor_id: None,
        status: None,
        desc: ComponentDescription::None,
    })
}
