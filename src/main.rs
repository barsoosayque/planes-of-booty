mod config;
mod state;
mod utils;

use bevy::{
    log::{Level, LogSettings},
    prelude::*,
};
use bevy_fallible::FallibleSystemPlugin;
use state::StatePlugin;

pub use config::Config;

#[bevy_main]
fn main() {
    fn startup(config: Res<Config>) {
        let mode = build_type!(dev: "dev", prod: "prod");
        info!("Running {} v{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), mode);

        build_type!(dev: { info!("{:?}", *config); });
    }

    App::build()
        .add_resource(Config::from_env())
        .add_resource(LogSettings { level: Level::DEBUG, ..LogSettings::default() })
        .add_resource(ClearColor(Color::rgb(0.08, 0.04, 0.1)))
        .add_plugins(DefaultPlugins)
        .add_plugin(StatePlugin)
        .add_plugin(FallibleSystemPlugin)
        .add_startup_system(startup.system())
        .run();
}
