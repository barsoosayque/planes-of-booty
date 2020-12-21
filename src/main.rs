mod state;
mod utils;

use bevy::{
    log::{Level, LogSettings},
    prelude::*,
};
use bevy_fallible::FallibleSystemPlugin;
use state::StatePlugin;

fn main() {
    fn startup() {
        info!("Running {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    }

    App::build()
        .add_resource(LogSettings { level: Level::DEBUG, ..LogSettings::default() })
        .add_plugins(DefaultPlugins)
        .add_plugin(StatePlugin)
        .add_plugin(FallibleSystemPlugin)
        .add_startup_system(startup.system())
        .run();
}
