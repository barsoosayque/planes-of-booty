#![allow(warnings)]
use crate::ecs::{component, resource, tag};

// see `build/build.rs` for code generation
include!(concat!(env!("OUT_DIR"), "/generated/item.rs"));

pub trait Consumable {
    fn description(&self) -> &str;
    fn consume(&self);
}

struct Healing;
impl Consumable for Healing {
    fn description(&self) -> &str { "Heals for 50 points in 3 seconds." }

    fn consume(&self) {}
}
