#![allow(warnings)]
use crate::ecs::resource;
use crate::{entity, item};

// see `build/build.rs` for code generation
include!(concat!(env!("OUT_DIR"), "/generated/arena.rs"));
include!(concat!(env!("OUT_DIR"), "/generated/spawn_group.rs"));
