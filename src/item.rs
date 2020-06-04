#![allow(warnings)]
use crate::{
    assets::*,
    ecs::{component, resource, tag},
};
use specs::{Entity, LazyUpdate, WorldExt};
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

struct Pitaya;
impl ConsumeBehaviour for Pitaya {
    fn description(&self) -> &str { "Deals 2x damage for the next 10 seconds." }

    fn icon(&self, ctx: &mut ggez::Context, assets: &mut AssetManager) -> Option<Arc<ImageAsset>> {
        assets.get::<ImageAsset>("/sprites/ui/attack.png", ctx).ok()
    }

    fn update(&self, dt: f32, time: f32, e: Entity, update: &LazyUpdate) -> bool {
        if (time - dt == 0.0) {
            update.exec(move |world| {
                if let Some(weaponry) = world.write_storage::<component::Weaponry>().get_mut(e) {
                    weaponry.damage_multiplier *= 2.0;
                }
            });
        } else if time >= 10.0 {
            update.exec(move |world| {
                if let Some(weaponry) = world.write_storage::<component::Weaponry>().get_mut(e) {
                    weaponry.damage_multiplier /= 2.0;
                }
            });
        }
        time >= 10.0
    }
}

struct Coconut;
impl ConsumeBehaviour for Coconut {
    fn description(&self) -> &str { "Increased speed and control for the next 20 seconds." }

    fn icon(&self, ctx: &mut ggez::Context, assets: &mut AssetManager) -> Option<Arc<ImageAsset>> {
        assets.get::<ImageAsset>("/sprites/ui/speed.png", ctx).ok()
    }

    fn update(&self, dt: f32, time: f32, e: Entity, update: &LazyUpdate) -> bool {
        if (time - dt == 0.0) {
            update.exec(move |world| {
                if let Some(movement) = world.write_storage::<component::Movement>().get_mut(e) {
                    movement.max_velocity *= 1.5;
                    movement.acceleration_flat *= 2.0;
                    movement.steering_difficulty -= 0.07;
                }
            });
        } else if time >= 20.0 {
            update.exec(move |world| {
                if let Some(movement) = world.write_storage::<component::Movement>().get_mut(e) {
                    movement.max_velocity /= 1.5;
                    movement.acceleration_flat /= 2.0;
                    movement.steering_difficulty += 0.07;
                }
            });
        }
        time >= 20.0
    }
}

struct Starfruit;
impl ConsumeBehaviour for Starfruit {
    fn description(&self) -> &str { "Nothing can stop you now.\n(For at least 10 seconds)" }

    fn icon(&self, ctx: &mut ggez::Context, assets: &mut AssetManager) -> Option<Arc<ImageAsset>> {
        assets.get::<ImageAsset>("/sprites/ui/uber.png", ctx).ok()
    }

    fn update(&self, dt: f32, time: f32, e: Entity, update: &LazyUpdate) -> bool {
        if (time - dt == 0.0) {
            update.exec(move |world| {
                if let Some(dmg_reciever) = world.write_storage::<component::DamageReciever>().get_mut(e) {
                    for damage_type in &component::DAMAGE_TYPES {
                        dmg_reciever.update_immunity(*damage_type, 10.0);
                    }
                }
            });
        }
        time >= 10.0
    }
}
