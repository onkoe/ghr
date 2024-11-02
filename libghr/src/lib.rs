//! # `libghr`
//!
//! A library that creates a report of system information, including hardware and configuration.
//!
//! ## Building
//!
//! Nothing too significant here. Note that `libghr` uses `mold` by default as
//! its linker on Linux x86_64. You'll need that (or to delete `libghr/.cargo`)
//! to compile without big explosions.
//!
//! Also, for getting GPU information, we use the NVIDIA's `nvml` library, as
//! there is no open-source library to get their GPU information. When not
//! present, you may see an error message, but this can safely be ignored.

// Make sure we're on a supported operating system.
#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "linux"),
    not(target_os = "freebsd"),
    not(target_os = "android")
))]
compile_error!(
    "`libghr`: The target operating system is unsupported. Please \
see the project README for additional information."
);

pub mod error;
pub mod prelude;
pub mod report;

// re-export the prelude to the crate root
pub use crate::prelude::public::*;
