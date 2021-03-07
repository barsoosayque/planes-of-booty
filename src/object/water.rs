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
    commands: &mut Commands,
    query: Query<(Entity, &WaterObjectDef)>,
    pipeline: Res<WaterRenderPipeline>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>
) {
    for (entity, object) in query.iter() {
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 6.0 })),
                render_pipelines: RenderPipelines::from_handles(vec![&pipeline.descriptor]),
                ..Default::default()
            })
            .with(water_materials.add(WaterMaterial {
                color: object.color,
                water_texture: pipeline.default_texture.as_weak(),
                ..Default::default()
            }))
            .despawn(entity);
        // TODO: can't reuse an entity here, probably because it requires position-less component events
        // .insert(entity, PbrBundle {
        //     mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
        //     render_pipelines: RenderPipelines::from_handles(vec![&pipeline.0]),
        //     ..Default::default()
        // })
        // .insert_one(entity, water_materials.add(WaterMaterial { color: object.color }))
        // .remove_one::<WaterObjectDef>(entity);

        debug!("WaterObject spawned");
    }
}
