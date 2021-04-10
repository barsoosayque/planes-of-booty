use bevy::prelude::*;

mod camera;
mod ship;
mod water;

pub use camera::CameraObjectDef;
pub use ship::ShipObjectDef;
pub use water::WaterObjectDef;

/// TODO: [ERGO] the whole struct should be moved to data once
/// storing components in files is ergonomic enough
pub struct ObjectSpawners;

impl ObjectSpawners {
    pub fn systems() -> SystemSet {
        SystemSet::new()
            .with_system(camera::spawner_system.system())
            .with_system(water::spawner_system.system())
            .with_system(ship::spawner_system.system())
    }
}
