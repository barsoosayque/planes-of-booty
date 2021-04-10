use bevy::prelude::*;
use typed_builder::TypedBuilder;

const BASIS_DISTANCE: f32 = 10.0;

/// High-level description of Camera object.
///
/// Use `CameraObjectDef::builder()` to build an instance
#[derive(TypedBuilder, Debug)]
pub struct CameraObjectDef {
    target: Entity,

    #[builder(default = 45.0)]
    angle: f32,

    #[builder(default = 1.0)]
    zoom: f32,
}

/// System that converts Camera object definitions into game objects
// TODO: [PERF] Change query to Added when it become system independent
pub fn spawner_system(mut commands: Commands, query: Query<(Entity, &CameraObjectDef)>) {
    for (entity, object) in query.iter() {
        let distance = BASIS_DISTANCE * object.zoom;
        let radians = std::f32::consts::PI * (object.angle / 180.0);

        commands
            .entity(entity)
            .insert_bundle(PerspectiveCameraBundle {
                transform: Transform::from_xyz(
                    0.0,
                    distance * radians.sin(),
                    distance * radians.cos(),
                )
                .looking_at(Vec3::ZERO, Vec3::Y),
                ..Default::default()
            })
            .remove::<CameraObjectDef>();
    }
}
