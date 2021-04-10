use bevy::prelude::*;
use typed_builder::TypedBuilder;

/// High-level description of Ship object.
///
/// Use `ShipObjectDef::builder()` to build an instance
#[derive(TypedBuilder)]
pub struct ShipObjectDef {}

/// System that converts Ship object definitions into game objects
// TODO: [PERF] Change query to Added when it become system independent
pub fn spawner_system(
    mut commands: Commands,
    query: Query<(Entity, &ShipObjectDef)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, _) in query.iter() {
        commands
            .entity(entity)
            .insert_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::SEA_GREEN.into()),
                ..Default::default()
            })
            .remove::<ShipObjectDef>();

        debug!("ShipObject spawned");
    }
}
