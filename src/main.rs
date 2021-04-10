mod config;
mod object;
mod render;
mod state;
mod utils;
mod scripting;

use bevy::{
    log::{Level, LogSettings},
    prelude::*,
};
use render::RenderPlugin;
use state::StatePlugin;
use scripting::ScriptingPlugin;

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
        .insert_resource(Config::from_env())
        .insert_resource(LogSettings {
            level: Level::DEBUG,
            filter: "wgpu=error,bevy_megaui=error,bevy_app=info,bevy_egui=info".to_owned(),
        })
        .insert_resource(ClearColor(Color::rgb(0.08, 0.04, 0.1)))
        .add_plugins(DefaultPlugins)
        .add_plugin(ScriptingPlugin)
        .add_plugin(RenderPlugin)
        .add_plugin(StatePlugin)
        .add_startup_system(startup.system())
        .run();
}
