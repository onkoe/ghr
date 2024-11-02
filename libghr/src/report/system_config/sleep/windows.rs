use core::ffi::c_void;

use windows::Win32::{
    Foundation::{STATUS_ACCESS_DENIED, STATUS_BUFFER_TOO_SMALL, STATUS_SUCCESS},
    System::Power::{CallNtPowerInformation, POWER_INFORMATION_LEVEL, SYSTEM_POWER_CAPABILITIES},
};

use crate::prelude::internal::*;

/// gets info about the computer's sleep states.
#[tracing::instrument]
pub(super) async fn get() -> Sleep {
    let mut sleep = Sleep::default();

    // no help from `wmi` here. let's just ask win32 manually...
    //
    // set the info level to the POWER_INFORMATION_LEVEL::SystemPowerCapabilities variant
    let info_magic_number = 4;
    let info_level = POWER_INFORMATION_LEVEL(info_magic_number);

    // the input buffer must be null.
    let input_buffer = None;
    let input_buffer_length = 0_u32;

    let mut output_buffer: SYSTEM_POWER_CAPABILITIES = Default::default();

    // SAFETY: only if the given `NTSTATUS` is good do we use the output buf.
    //
    // in addition, we follow the instructions as given in the win32 docs:
    // https://learn.microsoft.com/en-us/windows/win32/api/powerbase/nf-powerbase-callntpowerinformation
    let result = unsafe {
        CallNtPowerInformation(
            info_level,
            input_buffer,
            input_buffer_length,
            Some(core::ptr::addr_of_mut!(output_buffer) as *mut c_void),
            core::mem::size_of::<SYSTEM_POWER_CAPABILITIES>() as u32,
        )
    };

    // report any errors
    if result == STATUS_BUFFER_TOO_SMALL {
        tracing::error!(
            "`win32` failed to fill the sleep buffer since it was \
        too small. This is a `libghr` bug - please report it!"
        );
    } else if result == STATUS_ACCESS_DENIED {
        tracing::warn!("We are not permitted to access device power information.");
    }

    // check that the status was otherwise good
    if result == STATUS_SUCCESS {
        // we can read the output buffer.
        //
        // let's start with the basic sleep states
        sleep.s1 = output_buffer.SystemS1.as_bool().into();
        sleep.s2 = output_buffer.SystemS2.as_bool().into();
        sleep.s3 = output_buffer.SystemS3.as_bool().into();
        sleep.s4 = output_buffer.SystemS4.as_bool().into();

        // and here's s0ix
        sleep.s0ix = output_buffer.AoAc.as_bool().into();

        // note: windows doesn't have a direct equivalent to the linux software
        // suspend state (s0). so, by default, it's unknown.
    }

    sleep
}
