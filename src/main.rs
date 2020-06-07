use anyhow::Result;
use ggez::{
    conf::{FullscreenType, NumSamples, WindowMode, WindowSetup},
    event, ContextBuilder,
};
use log::info;

#[macro_use]
mod math;

mod arena;
mod assets;
mod attack;
mod ecs;
mod entity;
mod game;
mod item;
mod main_menu;
mod particle;
mod scene;
mod shader;
mod ui;

pub fn setup_logging() -> Result<()> {
    use fern::colors::{Color, ColoredLevelConfig};
    let colors = ColoredLevelConfig::default().info(Color::Blue).debug(Color::Green).trace(Color::Magenta);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!("[{:<5}][{}] {}", colors.color(record.level()), record.target(), message))
        })
        // Filter out unnecessary stuff
        .level(if std::env::args().any(|a| a == "--debug") { log::LevelFilter::Trace } else { log::LevelFilter::Info })
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
        .window_mode(WindowMode { fullscreen_type: FullscreenType::True, borderless: true, ..WindowMode::default() })
        .add_resource_path("resources")
        .build()?;

    let mut scene_manager = scene::SceneManager::new();
    scene_manager.send_command(scene::SceneCommand::Push(|ctx| Box::new(main_menu::MainMenu::new(ctx))));
    event::run(&mut ctx, &mut event_loop, &mut scene_manager).map_err(|err| anyhow::Error::new(err))
}

fn main() {
    setup_logging().expect("Loggin intializtion error");
    match run() {
        Ok(_) => info!("Exited cleanly."),
        Err(e) => info!("Error occured: {}", e),
    }
}
