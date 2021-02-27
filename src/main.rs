mod config;
mod render;
mod state;
mod utils;
mod object;

use bevy::{
    log::{Level, LogSettings},
    prelude::*,
};
use bevy_fallible::FallibleSystemPlugin;
use state::StatePlugin;
use render::RenderPlugin;
use object::ObjectPlugin;

pub use config::Config;

#[bevy_main]
fn main() {
    fn startup(config: Res<Config>, asset_server: ResMut<AssetServer>) {
        let mode = build_type!(dev: "dev", prod: "prod");
        info!("Running {} v{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), mode);

        build_type!(dev: {
            debug!("{:#?}", *config);
        });
    }

    App::build()
        .add_resource(Config::from_env())
        .add_resource(LogSettings {
            level: Level::DEBUG,
            filter: "wgpu=error,bevy_megaui=error,bevy_app=info,bevy_egui=info".to_owned(),
        })
        .add_resource(ClearColor(Color::rgb(0.08, 0.04, 0.1)))
        .add_plugins(DefaultPlugins)
        .add_plugin(ObjectPlugin)
        .add_plugin(RenderPlugin)
        .add_plugin(StatePlugin)
        .add_plugin(FallibleSystemPlugin)
        .add_startup_system(startup.system())
        .run();
}
