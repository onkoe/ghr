use std::sync::Arc;

use futures::StreamExt;
use nvml_wrapper::{enum_wrappers::device::Clock, enums::device::BusType, error::NvmlError, Nvml};

use crate::prelude::internal::*;

#[tracing::instrument]
pub(crate) async fn get() -> GhrResult<Vec<ComponentInfo>> {
    // attempt to load the nvidia `nvml` library. this allows access to
    // the values of gpu specs + performance.
    let nvml = Nvml::init().map_err(|e| {
        tracing::error!(
            "Failed to initialize the NVML driver, so cannot get \
            info for NVIDIA cards on the official driver. \n
            Please make sure this library is present on the system. \
            (err: {e})"
        );
        GhrError::ComponentInfoInaccessible(format!(
            "The NVIDIA `nvml` library was unavailable. (err: {e})"
        ))
    })?;

    // the driver loaded successfully. let's grab all devices.
    loop_on_devices(nvml).await.map_err(|e| {
        tracing::error!("Failed to get number of NVIDIA devices from `nvml`. (err: {e})");
        GhrError::ComponentInfoInaccessible(format!(
            "Failed to get number of NVIDIA devices from `nvml`. (err: {e})"
        ))
    })
}

#[tracing::instrument(skip(nvml))]
/// iterates over each discovered devices, then returns any componentinfos
/// found.
async fn loop_on_devices(nvml: Nvml) -> Result<Vec<ComponentInfo>, NvmlError> {
    // put the `nvml` in an `Arc` to allow cross-thread borrows
    let nvml = Arc::new(nvml);

    // grab the number of devices
    let device_count = nvml.device_count()?;

    // make a new future for each device
    let mut futures = futures::stream::iter((0..device_count).map(|device_id| {
        let local_nvml = Arc::clone(&nvml);
        blocking::unblock(move || check_device(local_nvml, device_id))
    }))
    .buffer_unordered(2);

    // get all devices
    let mut devices = Vec::new();
    while let Some(Some(dev)) = futures.next().await {
        devices.push(dev);
    }

    Ok(devices)
}

#[tracing::instrument(skip(nvml))]
/// makes a `ComponentInfo` for the device, if applicable
fn check_device(nvml: Arc<Nvml>, device_id: u32) -> Option<ComponentInfo> {
    // grab the device at the given index
    let device = nvml.device_by_index(device_id);

    // make sure it actually worked
    let Ok(device) = device else {
        tracing::warn!("Failed to access NVIDIA device with index `{device_id}`.");
        return None;
    };

    // it did! let's grab some info about it
    let clock_speed = device
        .max_clock_info(Clock::Graphics)
        .trace_ok("clock speed", device_id);
    let video_memory = device
        .memory_info()
        .trace_ok("video memory", device_id)
        .map(|v| unit_to_mibiunits(v.total));
    let video_memory_speed = device
        .max_clock_info(Clock::Memory)
        .trace_ok("video memory speed", device_id);

    // make a struct for that specific info
    let gpu_desc = GpuDescription {
        clock_speed,
        video_memory,
        video_memory_speed,
    };

    // now let's grab some general info
    let bus = device.bus_type().ok().and_then(|bt|
        match bt {
            BusType::Unknown => None,
            BusType::Pci => Some(ComponentBus::Pci),
            BusType::Pcie => Some(ComponentBus::Pcie),
            BusType::Fpci => Some(ComponentBus::Fpci),
            BusType::Agp => Some(ComponentBus::Agp),
        })
        .unwrap_or_else(|| {
            tracing::warn!("Failed to find component bus for NVIDIA card with index {device_id}. Assuming PCI.");
            ComponentBus::Pci
        });

    let id = device.name().ok();
    let class = None; // TODO(bray): you can probably get this somewhere
    let vendor_id = device.brand().ok().map(|b| format!("NVIDIA ({b:?})"));
    let status = None; // TODO

    Some(ComponentInfo {
        bus,
        id,
        class,
        vendor_id,
        status,
        desc: ComponentDescription::GpuDescription(gpu_desc),
    })
}

#[allow(dead_code)] // it aint dead on windows. yet.
trait TraceNvmlError<T> {
    fn trace_ok(self, id: impl AsRef<str>, device_id: u32) -> Option<T>;
}

impl<T> TraceNvmlError<T> for Result<T, NvmlError> {
    #[tracing::instrument(skip(self, id))]
    fn trace_ok(self, id: impl AsRef<str>, device_id: u32) -> Option<T> {
        match self {
            Ok(t) => Some(t),
            Err(e) => {
                tracing::warn!(
                    "Failed to get {} value for NVIDIA device with ID `{device_id}`. (err: {e})",
                    id.as_ref()
                );
                None
            }
        }
    }
}
