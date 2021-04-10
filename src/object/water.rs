use crate::render::{WaterMaterial, WaterRenderPipeline};
use bevy::prelude::*;
use typed_builder::TypedBuilder;

/// High-level description of Water object.
///
/// Use `WaterObjectDef::builder()` to build an instance
#[derive(TypedBuilder)]
pub struct WaterObjectDef {
    #[builder(default = Color::rgb(0.61, 0.86, 0.91))]
    color: Color,
}

/// System that converts Water object definitions into game objects
// TODO: [PERF] Change query to Added when it become system independent
pub fn spawner_system(
    mut commands: Commands,
    query: Query<(Entity, &WaterObjectDef)>,
    pipeline: Res<WaterRenderPipeline>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
) {
    for (entity, object) in query.iter() {
        commands
            .entity(entity)
            .insert_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 6.0 })),
                render_pipelines: RenderPipelines::from_handles(vec![&pipeline.descriptor]),
                visible: Visible { is_transparent: true, ..Default::default() },
                ..Default::default()
            })
            .insert(water_materials.add(WaterMaterial {
                color: object.color,
                water_texture: pipeline.default_texture.clone(),
                ..Default::default()
            }))
            .remove::<WaterObjectDef>();

        debug!("WaterObject spawned");
    }
}
