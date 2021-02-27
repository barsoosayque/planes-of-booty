use bevy::prelude::*;

mod water;

pub use water::WaterObject;

mod stage {
    pub const SPAWNER: &'static str = "object-spawner";
}
pub struct ObjectPlugin;
impl Plugin for ObjectPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let stage = SystemStage::parallel().with_system(water::spawner_system.system());
        app.add_stage_before(bevy::prelude::stage::PRE_UPDATE, stage::SPAWNER, stage);
    }
}
