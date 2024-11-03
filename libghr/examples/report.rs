//! Just makes a report.
//!
//! This module uses `tracing-flame` to generate a flamegraph of the library's
//! execution. There are three steps to get it working:
//!
//! 1. `cargo binstall inferno`
//! 2. `cargo run --example report --release`
//! 3. `cat tracing.folded | inferno-flamegraph > tracing-flamegraph.svg`
//!
//! You may also use `cargo-flamegraph` to get a more detailed view of the
//! situation. Here are the instructions for that:
//!
//! 1. `cargo binstall flamegraph`
//! 2. Depending on your platform:
//!     - Windows: `$env:CARGO_PROFILE_RELEASE_DEBUG=true; cargo flamegraph --example report --release`
//!     - Linux: `CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --example report --release`
//!
//! On Windows, you MUST run that command in an elevated command prompt - you'll
//! get some crazy errors otherwise.
//!
//! You may also wish to use `dtrace` on Windows if your debugger isn't working
//! as you'd expect.
//!
//! Here's the instructions for that: https://github.com/microsoft/DTrace-on-Windows
//!
//! By the way, if you're not getting debug symbols, make sure Windows Defender
//! has its defintions updated.
//!
//! See: https://github.com/flamegraph-rs/flamegraph/issues/338
//!
//! Finally, you might want to add the following to `ghr/Cargo.toml` if you're
//! having trouble:
//!
//! ```toml
//! [profile.release]
//! debug = true
//! strip = false
//! ```

use libghr::report::Report;
use tokio::time::Instant;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

#[tokio::main]
#[tracing::instrument]
async fn main() {
    // init tracing_flame
    setup_global_subscriber();

    // start timer
    let start = Instant::now();

    // make the report
    let _ = Report::new().await.unwrap();
    println!("{} ns", Instant::now().duration_since(start).as_nanos());
}

#[tracing::instrument]
fn setup_global_subscriber() -> impl Drop {
    let fmt_layer = tracing_subscriber::fmt::Layer::default();

    let (flame_layer, _guard) = tracing_flame::FlameLayer::with_file("./tracing.folded").unwrap();

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(flame_layer)
        .init();
    _guard
}
