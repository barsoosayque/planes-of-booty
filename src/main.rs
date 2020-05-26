use anyhow::Result;
use ggez::{
    conf::{NumSamples, WindowSetup},
    event, ContextBuilder,
};
use log::info;

#[macro_use]
mod math;

mod attack;
mod assets;
mod ecs;
mod entity;
mod game;
mod item;
mod ui;

pub fn setup_logging() -> Result<()> {
    use fern::colors::{Color, ColoredLevelConfig};
    let colors = ColoredLevelConfig::default().info(Color::Blue).debug(Color::Green).trace(Color::Magenta);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{:<5}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        // Filter out unnecessary stuff
        .level(log::LevelFilter::Trace)
        .level_for("gfx", log::LevelFilter::Off)
        .level_for("gfx_device_gl", log::LevelFilter::Off)
        .level_for("gilrs", log::LevelFilter::Off)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

fn run() -> Result<()> {
    info!("Running {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let (mut ctx, mut event_loop) = ContextBuilder::new("planes-of-booty", "")
        .window_setup(WindowSetup {
            title: "Planes of Booty".to_owned(),
            samples: NumSamples::Four,
            vsync: true,
            ..WindowSetup::default()
        })
        .add_resource_path("resources")
        .build()?;

    let mut instance = game::Game::new(&mut ctx);
    event::run(&mut ctx, &mut event_loop, &mut instance).map_err(|err| anyhow::Error::new(err))
}

fn main() {
    setup_logging().expect("Loggin intializtion error");
    match run() {
        Ok(_) => info!("Exited cleanly."),
        Err(e) => info!("Error occured: {}", e),
    }
}
