use bevy::prelude::*;
use fern::colors::{Color, ColoredLevelConfig};
use bevy_fallable::fallable_system;
use anyhow::Result;

pub struct LoggingPlugin;

impl Plugin for LoggingPlugin {
    fn build(&self, app: &mut AppBuilder) { app.add_startup_system(Self::setup.system()); }
}

impl LoggingPlugin {
    #[fallable_system]
    fn setup() -> Result<()> {
        let colors = ColoredLevelConfig::default().info(Color::Blue).debug(Color::Green).trace(Color::Magenta);
        let level =
            if std::env::args().any(|a| a == "--debug") { log::LevelFilter::Trace } else { log::LevelFilter::Info };

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
}
