use crate::ecs::{component, tag};

// see `build.rs` for entity code generation
include!(concat!(env!("OUT_DIR"), "/entity_gen.rs"));
