#![allow(warnings)]
use crate::ecs::{component, resource, tag};

// see `build/build.rs` for code generation
include!(concat!(env!("OUT_DIR"), "/generated/item.rs"));

pub trait ConsumeBehaviour: Sync + Send {
    fn description(&self) -> &str;
    fn update(&self, dt: f32, time: f32) -> bool;
}

struct Orange;
impl ConsumeBehaviour for Orange {
    fn description(&self) -> &str { "Heals for 50 points in 3 seconds." }

    fn update(&self, dt: f32, time: f32) -> bool {
        true
    }
}
