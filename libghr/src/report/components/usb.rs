//! i am usb

use super::{ComponentBus, ComponentInfo, ComponentStatus};
use crate::prelude::internal::*;

use tokio::try_join;

#[cfg(target_os = "linux")]
pub async fn usb_components() -> GhrResult<Vec<ComponentInfo>> {
    // grab info about usb devices and construct reprs
    let mut usb = Vec::new();
    for dev in super::linux::devices("/sys/bus/usb/devices").await? {
        // grab the component's path
        let path = dev.path();

        // read a few files to get important info about this thang
        let Ok((id, vendor_id, class)) = try_join!(
            tokio::fs::read_to_string(path.join("idProduct")),
            tokio::fs::read_to_string(path.join("idVendor")),
            tokio::fs::read_to_string(path.join("bInterfaceClass")),
        ) else {
            tracing::warn!(
                "A USB component's info was inaccessible! (device at `{})`",
                path.display()
            );
            continue;
        };

        usb.push(ComponentInfo {
            bus: ComponentBus::Usb,
            id,
            class,
            vendor_id,
            status: ComponentStatus {}, // TODO
        })
    }

    Ok(usb)
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
                id: id.to_string_lossy().to_string(),
                class: class.to_string_lossy().to_string(),
                vendor_id: vendor_id.to_string_lossy().to_string(),
                status: ComponentStatus {}, // TODO
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
