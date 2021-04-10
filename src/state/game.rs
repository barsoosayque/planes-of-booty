use super::{State, States};
use crate::object::{CameraObjectDef, ObjectSpawners, ShipObjectDef, WaterObjectDef};
use bevy::prelude::*;
// use lazy_static_include::lazy_static_include_str;

// TODO: [ERGO] asset manager
// lazy_static_include_str! {
//     GAME_INIT_SCRIPT => "assets/scripts/game_init.rune",
// }

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // Game
            .add_system_set(SystemSet::on_enter(State::Game).with_system(setup.system()))
            .add_system_set(
                ObjectSpawners::systems().with_run_criteria(States::on_update(State::Game)),
            );
    }
}

fn setup(mut commands: Commands) {
    // spawn scene
    commands.spawn().insert(WaterObjectDef::builder().build());
    let ship_entity = commands.spawn().insert(ShipObjectDef::builder().build()).id();

    commands
        .spawn()
        .insert(CameraObjectDef::builder().zoom(1.0).angle(60.0).target(ship_entity).build());

    // light
    commands.spawn().insert_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..Default::default()
    });
}
