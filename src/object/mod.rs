use bevy::{ecs::ShouldRun, prelude::*};

mod camera;
mod ship;
mod water;

pub use camera::CameraObjectDef;
pub use ship::ShipObjectDef;
pub use water::WaterObjectDef;

mod stage {
    pub const SPAWNER: &'static str = "object-spawner";
}

/// FIXME: use states when sysetms sets are merged
pub struct ObjectSpanwersEnabler(pub bool);

/// TODO: the whole plugin should be moved to data once
/// storing components in files is ergonomic enoughkj
pub struct ObjectPlugin;
impl Plugin for ObjectPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let stage = SystemStage::parallel()
            .with_system(camera::spawner_system.system())
            .with_system(water::spawner_system.system())
            .with_system(ship::spawner_system.system())
            .with_run_criteria(run_spawners_if_enabled_system.system());

        app.add_resource(ObjectSpanwersEnabler(false)).add_stage_before(
            bevy::prelude::stage::PRE_UPDATE,
            stage::SPAWNER,
            stage,
        );
    }
}

fn run_spawners_if_enabled_system(enabler: Res<ObjectSpanwersEnabler>) -> ShouldRun {
    if enabler.0 {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}
