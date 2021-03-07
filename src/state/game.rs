use crate::object::{CameraObjectDef, ObjectSpanwersEnabler, ShipObjectDef, WaterObjectDef};
use bevy::prelude::*;

pub fn setup(commands: &mut Commands, mut enabler: ResMut<ObjectSpanwersEnabler>) {
    enabler.0 = true;

    // spawn scene
    commands.spawn((WaterObjectDef::builder().build(),)).spawn((ShipObjectDef::builder().build(),));

    commands
        .spawn((CameraObjectDef::builder().target(commands.current_entity().unwrap()).build(),))
        // light
        .spawn(LightBundle { transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)), ..Default::default() });
}
