mod water;

use bevy::prelude::*;
pub use water::{WaterMaterial, WaterRenderPipeline};

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<WaterMaterial>().add_startup_system(water::setup_pipeline.system());
    }
}
