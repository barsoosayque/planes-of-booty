#[allow(unused_imports, unused_variables)]
use crate::ecs::{component, resource, tag};

// see `build/build.rs` for code generation
include!(concat!(env!("OUT_DIR"), "/generated/particle.rs"));
