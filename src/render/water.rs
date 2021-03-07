use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::PipelineDescriptor,
        render_graph::{base::node::MAIN_PASS, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::ShaderStages,
        texture::AddressMode,
    },
};

use crate::build_type;

use super::PipelineStatus;

#[derive(RenderResources, TypeUuid, Default)]
#[uuid = "adec8d58-5db9-47de-bba3-c7354a7c467c"]
pub struct WaterMaterial {
    pub color: Color,
    pub time: f64,
    pub water_texture: Handle<Texture>,
}

pub struct WaterRenderPipeline {
    pub descriptor: Handle<PipelineDescriptor>,
    pub default_texture: Handle<Texture>,
}

pub fn setup_pipeline(
    commands: &mut Commands,
    asset_server: ResMut<AssetServer>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    build_type!(dev: asset_server.watch_for_changes().unwrap());

    commands.insert_resource(WaterRenderPipeline {
        descriptor: pipelines.add(PipelineDescriptor::default_config(ShaderStages {
            vertex: asset_server.load::<Shader, _>("shaders/water.vert"),
            fragment: Some(asset_server.load::<Shader, _>("shaders/water.frag")),
        })),
        default_texture: asset_server.load::<Texture, _>("textures/water.png"),
    });

    let node = render_graph.add_system_node("water_material", AssetRenderResourcesNode::<WaterMaterial>::new(true));
    render_graph.add_node_edge(node, MAIN_PASS).unwrap();
}

pub fn finalize_pipeline_system(
    mut state: Local<EventReader<AssetEvent<Texture>>>,
    pipeline: Res<WaterRenderPipeline>,
    mut textures: ResMut<Assets<Texture>>,
    mut status: ResMut<PipelineStatus>,
    texture_events: Res<Events<AssetEvent<Texture>>>,
) {
    for event in state.iter(&texture_events) {
        match event {
            AssetEvent::Created { handle } if handle == &pipeline.default_texture => {
                let mut texture = textures.get_mut(handle.clone_weak()).unwrap();
                texture.sampler.address_mode_u = AddressMode::Repeat;
                texture.sampler.address_mode_v = AddressMode::Repeat;
                texture.sampler.address_mode_w = AddressMode::Repeat;
                status.is_water_ready = true;
            },
            _ => {},
        }
    }
}

pub fn update_water_material_system(
    query: Query<&Handle<WaterMaterial>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    time: Res<Time>,
) {
    for handle in query.iter() {
        let material = water_materials.get_mut(handle).unwrap();
        material.time = time.seconds_since_startup();
    }
}
