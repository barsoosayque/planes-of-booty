use crate::render::{WaterMaterial, WaterRenderPipeline};
use bevy::prelude::*;
use typed_builder::TypedBuilder;

/// High-level description of Water object.
///
/// Use `WaterObject::builder()` to build an instance
#[derive(TypedBuilder)]
pub struct WaterObject {
    #[builder(default = Color::rgb(0.61, 0.86, 0.91))]
    color: Color,
}

/// System that converts WaterObject definitions into game objects
// TODO: [PERF] Change query to Added when it become system independent
pub fn spawner_system(
    commands: &mut Commands,
    query: Query<(Entity, &WaterObject)>,
    pipeline: Res<WaterRenderPipeline>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
) {
    for (entity, object) in query.iter() {
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
                render_pipelines: RenderPipelines::from_handles(vec![&pipeline.0]),
                ..Default::default()
            })
            .with(water_materials.add(WaterMaterial { color: object.color }))
            .despawn(entity);
    }
}
