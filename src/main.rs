use anyhow::Result;
use bevy::prelude::*;
use log::info;

fn run() { App::build().run(); }

pub fn setup_logger() -> Result<()> {
    use fern::colors::{Color, ColoredLevelConfig};
    let colors = ColoredLevelConfig::default().info(Color::Blue).debug(Color::Green).trace(Color::Magenta);
    let level = if std::env::args().any(|a| a == "--debug") { log::LevelFilter::Trace } else { log::LevelFilter::Info };
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!("[{:<5}][{}] {}", colors.color(record.level()), record.target(), message))
        })
        // Filter out unnecessary stuff
        .level(level)
        .level_for("gfx", log::LevelFilter::Off)
        .level_for("gfx_device_gl", log::LevelFilter::Off)
        .level_for("gilrs", log::LevelFilter::Off)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

fn main() {
    setup_logger().expect("Logger intializtion error");
    info!("Running {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    run();
    info!("Exited cleanly.");
}
