// Just makes a report.
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
