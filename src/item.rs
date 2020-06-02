#![allow(warnings)]
use crate::ecs::{component, resource, tag};
use specs::{Entity, LazyUpdate, WorldExt};
use crate::assets::*;
use std::sync::Arc;

// see `build/build.rs` for code generation
include!(concat!(env!("OUT_DIR"), "/generated/item.rs"));

pub trait ConsumeBehaviour: Sync + Send {
    fn description(&self) -> &str;
    fn icon(&self, ctx: &mut ggez::Context, assets: &mut AssetManager) -> Option<Arc<ImageAsset>>;
    fn update(&self, dt: f32, time: f32, e: Entity, update: &LazyUpdate) -> bool;
}

struct Orange;
impl ConsumeBehaviour for Orange {
    fn description(&self) -> &str { "Heals 20 HP points every seconds for 3 seconds." }

    fn icon(&self, ctx: &mut ggez::Context, assets: &mut AssetManager) -> Option<Arc<ImageAsset>> {
        assets.get::<ImageAsset>("/sprites/ui/healing.png", ctx).ok()
    }

    fn update(&self, dt: f32, time: f32, e: Entity, update: &LazyUpdate) -> bool {
        // we just crossed second mark
        if (time - dt == 0.0) || time.floor() != (time - dt).floor() {
            update.exec(move |world| {
                if let Some(hpool) = world.write_storage::<component::HealthPool>().get_mut(e) {
                    hpool.hp = (hpool.hp + 20).min(hpool.max_hp);
                }
            });
        }
        time >= 3.0
    }
}
