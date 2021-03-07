mod water;

use bevy::{ecs::ShouldRun, prelude::*};
pub use water::{WaterMaterial, WaterRenderPipeline};

mod stage {
    pub const FINALIZE_PIPELINE: &'static str = "finalize-pipeline";
}
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
        let stage = SystemStage::parallel()
            .with_system(water::finalize_pipeline_system.system())
            .with_run_criteria(run_finalizers_if_not_ready.system());

        app.init_resource::<PipelineStatus>()
            .add_asset::<WaterMaterial>()
            .add_startup_system(water::setup_pipeline.system())
            .add_stage_before(bevy::prelude::stage::PRE_UPDATE, stage::FINALIZE_PIPELINE, stage)
            .add_system(water::update_water_material_system.system());
    }
}
