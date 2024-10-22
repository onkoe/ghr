//! # `libghr`
//!
//! A library that creates a report of system information, including hardware and configuration.

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
