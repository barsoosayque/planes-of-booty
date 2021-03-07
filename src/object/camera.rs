use bevy::prelude::*;
use typed_builder::TypedBuilder;

/// High-level description of Camera object.
///
/// Use `CameraObjectDef::builder()` to build an instance
#[derive(TypedBuilder)]
pub struct CameraObjectDef {
    target: Entity,

    #[builder(default = 45.0)]
    angle: f32,

    #[builder(default = 1.0)]
    zoom: f32
}

/// System that converts Camera object definitions into game objects
// TODO: [PERF] Change query to Added when it become system independent
pub fn spawner_system(
    commands: &mut Commands,
    query: Query<(Entity, &CameraObjectDef)>,
) {
    for (entity, object) in query.iter() {
        commands
            .spawn(Camera3dBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 5.0, object.zoom * 5.0))
                    .looking_at(Vec3::default(), Vec3::unit_y()),
                ..Default::default()
            })
            .despawn(entity);
    }
}
