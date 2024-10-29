use std::path::Path;

use crate::prelude::internal::*;

// TODO: find out what all gpus on linux should provide.
//
// (see `asahi-drm` for a nice, readable impl)

#[tracing::instrument]
pub(super) async fn gpu(path: &Path) -> GhrResult<ComponentInfo> {
    // grab class, name, and vendor
    let (class, id, vendor_id) = {
        let civ = Civ::new(path).await;
        (civ.class, civ.id, civ.vendor)
    };

    Ok(ComponentInfo {
        bus: ComponentBus::Pci,
        id,
        class,
        vendor_id,
        status: None,
        desc: ComponentDescription::GpuDescription(GpuDescription {
            clock_speed: None,
            video_memory: None,
            video_memory_speed: None,
        }),
    })
}
