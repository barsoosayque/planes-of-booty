mod water;

use bevy::{ecs::schedule::ShouldRun, prelude::*};
pub use water::{WaterMaterial, WaterRenderPipeline};

mod stage {
    pub const FINALIZE_PIPELINE: &'static str = "finalize-pipeline";
}

// TODO: [ERGO] rewrite to ShouldRun chain
#[derive(Default)]
pub struct PipelineStatus {
    pub is_water_ready: bool,
}

impl PipelineStatus {
    pub fn is_ready(&self) -> bool { self.is_water_ready }
}

fn run_finalizers_if_not_ready(status: Res<PipelineStatus>) -> ShouldRun {
    if status.is_ready() {
        ShouldRun::No
    } else {
        ShouldRun::Yes
    }
}

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PipelineStatus>()
            // water
            .add_asset::<WaterMaterial>()
            .add_startup_system(water::setup_pipeline.system())
            .add_system(water::update_water_material_system.system())
            // finalization
            .add_stage_before(
                bevy::app::CoreStage::PreUpdate,
                stage::FINALIZE_PIPELINE,
                SystemStage::parallel(),
            )
            .add_system_set_to_stage(
                stage::FINALIZE_PIPELINE,
                SystemSet::new()
                    .with_system(water::finalize_pipeline_system.system())
                    .with_run_criteria(run_finalizers_if_not_ready.system()),
            );
    }
}
