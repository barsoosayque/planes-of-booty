use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::PipelineDescriptor,
        render_graph::{base::node::MAIN_PASS, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::ShaderStages,
    },
};

use crate::build_type;

#[derive(RenderResources, TypeUuid)]
#[uuid = "adec8d58-5db9-47de-bba3-c7354a7c467c"]
pub struct WaterMaterial {
    pub color: Color,
}

pub struct WaterRenderPipeline(pub Handle<PipelineDescriptor>);

pub fn setup_pipeline(
    commands: &mut Commands,
    asset_server: ResMut<AssetServer>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    build_type!(dev: asset_server.watch_for_changes().unwrap());

    let handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: asset_server.load::<Shader, _>("shaders/water.vert"),
        fragment: Some(asset_server.load::<Shader, _>("shaders/water.frag")),
    }));
    commands.insert_resource(WaterRenderPipeline(handle));

    let node = render_graph.add_system_node("water_material", AssetRenderResourcesNode::<WaterMaterial>::new(true));
    render_graph.add_node_edge(node, MAIN_PASS).unwrap();
}
