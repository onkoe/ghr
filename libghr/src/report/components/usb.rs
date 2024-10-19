//! i am usb

use crate::prelude::internal::*;

use std::path::Path;
use tokio::join;
use tokio::try_join;

#[cfg(target_os = "linux")]
pub async fn usb_components() -> GhrResult<Vec<ComponentInfo>> {
    // grab info about usb devices and construct reprs
    let mut usb = Vec::new();
    for dev in super::linux::devices("/sys/bus/usb/devices").await? {
        // grab the component's path
        let path = dev.path();

        // read a few files to get important info about this thang
        let ((vendor_id, id), class) = join! {
            usb_vendor_and_id(&path),
            usb_class(&path)
        };

        usb.push(ComponentInfo {
            bus: ComponentBus::Usb,
            id,
            class,
            vendor_id,
            status: None, // TODOComponentStatus {}
            desc: ComponentDescription::None,
        })
    }

    Ok(usb)
}

/// on Linux, this grabs the class codes for our devices.
#[cfg(target_os = "linux")]
#[tracing::instrument]
async fn usb_class(path: &Path) -> Option<String> {
    // read the files
    let Ok((class, subclass)) = try_join!(
        tokio::fs::read_to_string(path.join("bDeviceClass")),
        tokio::fs::read_to_string(path.join("bDeviceSubClass")),
    ) else {
        tracing::warn!(
            "Failed to find device class/subclass! Yielding 'Unknown' for these values."
        );
        return None;
    };

    let (class, subclass) = (class.trim(), subclass.trim());

    // try grabbing a class
    // look it up
    if let (Ok(parsed_class), Ok(parsed_subclass)) = (
        u8::from_str_radix(class, 16),
        u8::from_str_radix(subclass, 16),
    ) {
        if let Some(c) = usb_ids::SubClass::from_cid_scid(parsed_class, parsed_subclass) {
            return Some(format!("{} ({})", c.class().name(), c.name()));
        }
    }

    // return the raw ID if we must
    return Some(format!("{} ({})", class, subclass));
}

#[cfg(target_os = "linux")]
#[tracing::instrument]
async fn usb_vendor_and_id(path: &Path) -> (Option<String>, Option<String>) {
    // look for human-readable string repr

    use usb_ids::FromId as _;
    let human_readable = tokio::join! {
        tokio::fs::read_to_string(path.join("iManufacturer")),
        tokio::fs::read_to_string(path.join("iProduct")),
    };

    // return human-readable strings if they exist for us!
    if let (Ok(vendor), Ok(product)) = human_readable {
        tracing::debug!(
            "oh cool, we got human readable strings! vendor: `{vendor}`. product: {product}."
        );
        return (Some(vendor), Some(product));
    }

    // otherwise, grab lame number
    let (vend, prod) = tokio::join! {
        tokio::fs::read_to_string(path.join("idVendor")),
        tokio::fs::read_to_string(path.join("idProduct")),
    };

    // trim whitespaces (newlines)
    let (mut vend, prod) = (
        vend.map(|s| s.trim().to_string()),
        prod.map(|s| s.trim().to_string()),
    );

    // convert them if they exist.
    if let (Ok(ref num_vend), Ok(ref num_prod)) = (&vend, &prod) {
        tracing::debug!("lame numbers {}; {}", num_vend, num_prod);

        // try to convert them to human readable names
        let (parsed_vend, parsed_prod) = (
            u16::from_str_radix(num_vend, 16),
            u16::from_str_radix(num_prod, 16),
        );

        if let (Ok(parsed_vend), Ok(parsed_prod)) = (parsed_vend, parsed_prod) {
            tracing::debug!("parsed from str radix");
            if let Some(dev) = usb_ids::Device::from_vid_pid(parsed_vend, parsed_prod) {
                tracing::debug!("names");
                return (Some(dev.vendor().name().into()), Some(dev.name().into()));
            }

            // sometimes, the linux usb list is incomplete. let's try just the vendor!
            if let Some(readable_vend) = usb_ids::Vendor::from_id(parsed_vend) {
                vend = Ok(readable_vend.name().to_string());
            }
        }
    }

    // otherwise, report errors and return "Unknown" strings
    if let Err(prod_err) = &prod {
        tracing::warn!(
            "Couldn't find product for USB device at path {path:?}. (error: {prod_err})"
        );
    }
    if let Err(vend_err) = &vend {
        tracing::warn!("Couldn't find vendor for USB device at path {path:?}. (error: {vend_err})");
    }

    // give up
    (None, None)
}

#[cfg(target_os = "windows")]
pub async fn usb_components() -> GhrResult<Vec<ComponentInfo>> {
    use std::{
        ffi::CString,
        ptr::{addr_of, addr_of_mut},
    };

    use ::windows::Win32::Foundation::{GetLastError, ERROR_NOT_FOUND};
    use bytemuck::{checked::try_cast, try_cast_ref};
    use windows::Win32::{
        Devices::DeviceAndDriverInstallation::{
            SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW,
            SetupDiGetDeviceInfoListDetailW, SetupDiGetDeviceInterfaceDetailW,
            SetupDiGetDeviceRegistryPropertyW, DIGCF_PRESENT, GUID_BUS_TYPE_USB,
            SETUP_DI_REGISTRY_PROPERTY, SPDRP_BUSNUMBER, SPDRP_CLASS, SPDRP_FRIENDLYNAME,
            SPDRP_MFG, SP_DEVICE_INTERFACE_DATA, SP_DEVINFO_DATA, SP_DEVINFO_LIST_DETAIL_DATA_W,
        },
        Foundation::HWND,
    };
    use windows_core::PCWSTR;

    // we want to use the USB type
    let usb_guid = GUID_BUS_TYPE_USB;

    // alright, we need to grab the device info set first
    let device_info_set = unsafe {
        SetupDiGetClassDevsW(
            Some(addr_of!(usb_guid)),
            PCWSTR::null(),
            HWND(std::ptr::null_mut()),
            DIGCF_PRESENT,
        )
    }
    .map_err(|e| GhrError::ComponentInfoInaccessible(e.to_string()))?;

    // grab device data until we can't anymore
    let mut all_device_interfaces: Vec<SP_DEVICE_INTERFACE_DATA> = Vec::new();
    let mut index = 0_u32;
    loop {
        let mut device_interface: SP_DEVICE_INTERFACE_DATA = unsafe { core::mem::zeroed() };
        unsafe {
            SetupDiEnumDeviceInterfaces(
                device_info_set,
                None,
                addr_of!(usb_guid),
                index,
                addr_of_mut!(device_interface),
            )
        }
        .map_err(|e| GhrError::ComponentInfoInaccessible(e.to_string()))?;

        // add it to the list
        all_device_interfaces.push(device_interface);

        // if the last device didn't exist, stop!
        if unsafe { GetLastError() } == ERROR_NOT_FOUND {
            break;
        }

        // increment the device index to grab the next one's info
        index += 1;
    }

    // we have our device interface! let's get some info about it
    let devices = all_device_interfaces
        .into_iter()
        .map(|device_interface_data| -> GhrResult<ComponentInfo> {
            // make it a `DEVINFO_DATA`
            let mut device_info_data: SP_DEVINFO_DATA = unsafe { core::mem::zeroed() };

            // pass to the func to fill it w/ needed info
            unsafe {
                SetupDiGetDeviceInterfaceDetailW(
                    device_info_set,
                    addr_of!(device_interface_data),
                    None,
                    0_u32,
                    None,
                    Some(addr_of_mut!(device_info_data)),
                )
            };

            // now we can use the DEVINFO_DATA to grab additional details about
            // the device!

            let get_prop =
                |prop: SETUP_DI_REGISTRY_PROPERTY, buf: &mut [u8]| -> windows_core::Result<()> {
                    unsafe {
                        SetupDiGetDeviceRegistryPropertyW(
                            device_info_set,
                            addr_of!(device_info_data),
                            SPDRP_MFG,
                            None,
                            Some(buf),
                            None,
                        )
                    }
                };

            let vendor_id = {
                let mut v = Box::new(Vec::with_capacity(512_usize));
                get_prop(SPDRP_MFG, v.as_mut_slice());
                CString::from_vec_with_nul(*v)
            }
            .map_err(|e| GhrError::ComponentInfoWeirdInfo(e.to_string()))?;

            let class = {
                let mut c = Box::new(Vec::with_capacity(512_usize));
                get_prop(SPDRP_CLASS, c.as_mut_slice());
                CString::from_vec_with_nul(*c)
            }
            .map_err(|e| GhrError::ComponentInfoWeirdInfo(e.to_string()))?;

            let id = {
                let mut i = Box::new(Vec::with_capacity(512_usize));
                get_prop(SPDRP_FRIENDLYNAME, i.as_mut_slice());
                CString::from_vec_with_nul(*i)
            }
            .map_err(|e| GhrError::ComponentInfoWeirdInfo(e.to_string()))?;

            // TODO: remove this if unused by PCI
            // let bus: i32 = {
            //     let mut b = [0_u8; 4];
            //     get_prop(SPDRP_BUSNUMBER, b.as_mut_slice());

            //     // use `bytemuck` to ensure we stay within alignment rules
            //     Ok(try_cast::<[u8; 4], i32>(b).map_err(|e| {
            //         tracing::error!("Woah, we've escaped the expected `repr(Rust)` alignment! Please report this as a bug!");
            //         GhrError::ComponentInfoWeirdInfo(e.to_string())
            //     })?)
            // }?;

            Ok(ComponentInfo {
                bus: ComponentBus::Usb,
                id: Some(id.to_string_lossy().to_string()),
                class: Some(class.to_string_lossy().to_string()),
                vendor_id: Some(vendor_id.to_string_lossy().to_string()),
                status: None, // TODO
                desc: ComponentDescription::None,
            })
        })
        .collect::<GhrResult<Vec<_>>>();

    // SAFETY: when we're done, we MUST call this function to drop the previous
    // list.
    unsafe {
        SetupDiDestroyDeviceInfoList(device_info_set).map_err(|e| {
            GhrError::ComponentInfoWeirdInfo(format!(
                "Couldn't gracefully destroy device info, which is required by Windows. See: {e}"
            ))
        })?;
    };

    Ok(devices?)
}
