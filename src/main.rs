mod state;
mod utils;

use bevy::prelude::*;
use bevy_fallable::FallableSystemPlugin;
use log::info;
use utils::LoggingPlugin;
use state::StatePlugin;

fn main() {
    info!("Running {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(StatePlugin)
        .add_plugin(FallableSystemPlugin)
        .add_plugin(LoggingPlugin)
        .run();

    info!("Exited cleanly.");
}
