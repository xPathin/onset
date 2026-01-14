mod app;
mod config;
mod desktop_entry;
mod discovery;
mod model;
mod operations;
mod ui;
mod utils;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "onset=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting onset");

    let exit_code = app::run();

    std::process::exit(exit_code);
}
