use crate::object::{CameraObjectDef, ObjectSpanwersEnabler, ShipObjectDef, WaterObjectDef};
use crate::scripting::RuneContext;
use bevy::prelude::*;
use lazy_static_include::lazy_static_include_str;

lazy_static_include_str! {
    GAME_INIT_SCRIPT => "assets/scripts/game_init.rune",
}

pub fn setup(
    commands: &mut Commands,
    mut rune: ResMut<RuneContext>,
    mut enabler: ResMut<ObjectSpanwersEnabler>,
) {
    enabler.0 = true;
    rune.run_script("game_init.rune", &GAME_INIT_SCRIPT);

    // spawn scene
    commands.spawn((WaterObjectDef::builder().build(),)).spawn((ShipObjectDef::builder().build(),));

    commands
        .spawn((CameraObjectDef::builder().target(commands.current_entity().unwrap()).build(),))
        // light
        .spawn(LightBundle { transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)), ..Default::default() });
}
