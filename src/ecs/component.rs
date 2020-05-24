use crate::{
    assets::*,
    math::{Direction, Size2f, Vec2f},
};
use nphysics2d::{
    ncollide2d::shape::ShapeHandle,
    object::{DefaultBodyHandle, DefaultColliderHandle},
};
use specs::{Component, Entity, FlaggedStorage, VecStorage, World, WorldExt};
use std::{collections::HashSet as Set, sync::Arc};

/////////////////////////
// Inventory and Items //
/////////////////////////

#[derive(Default, Debug, Component)]
#[storage(VecStorage)]
pub struct Inventory {
    pub content: Content,
}
#[derive(Default, Debug)]
pub struct Content(Vec<(Entity, u32)>);
impl Content {
    pub fn add(&mut self, world: &World, item: Entity, count: u32) {
        if count == 0 { return }
        let (reflections, stacks) = (world.read_storage::<Reflection>(), world.read_storage::<Stackable>());
        let stack_size = stacks.get(item).map(|s| s.stack_size).unwrap_or(1);
        let id = reflections.get(item).unwrap().id;

        let mut count_left = count;
        for (e, stack) in &mut self.0 {
            if reflections.get(*e).unwrap().id == id {
                let transfer_count = (stack_size - *stack).max(0).min(count_left);
                *stack += transfer_count;
                count_left -= transfer_count;
            }
            if count_left <= 0 {
                break;
            }
        }

        while count_left > 0 {
            let transfer_count = count_left.min(stack_size);
            self.0.push((item, transfer_count)); 
            count_left -= transfer_count;
        }
    }

    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn iter(&self) -> impl Iterator<Item=&(Entity, u32)> { self.0.iter() }
}

#[derive(Default, Debug, Component)]
#[storage(VecStorage)]
pub struct Named {
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Quality {
    pub rarity: Rarity,
}
#[derive(Debug)]
pub enum Rarity {
    Common,
    Rare,
    Epic,
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Stackable {
    pub stack_size: u32,
}
impl Default for Stackable {
    fn default() -> Self { Stackable { stack_size: 1 } }
}

/////////////
// Physics //
/////////////

#[derive(Component)]
#[storage(VecStorage)]
pub struct Physic {
    pub body: DefaultBodyHandle,
    pub collide: (DefaultColliderHandle, CollideShapeHandle),
}
pub type CollideShapeHandle = DirOrSingle<ShapeHandle<f32>>;

#[derive(Default, Debug, Component)]
#[storage(FlaggedStorage)]
pub struct Transform {
    pub pos: Vec2f,
    pub rotation: f32,
}

#[derive(Default, Debug, Component)]
#[storage(VecStorage)]
pub struct Movement {
    pub velocity: Vec2f,

    pub target_acceleration_normal: Vec2f,

    pub max_velocity: f32,
    pub acceleration_flat: f32,
    pub steering_difficulty: f32,
}

//////////////////////
// Targeting and AI //
//////////////////////

#[derive(Default, Debug, Component)]
#[storage(VecStorage)]
pub struct Target {
    pub target: Option<Entity>,
}

#[derive(Default, Component)]
#[storage(VecStorage)]
pub struct FollowTarget {
    pub keep_distance: f32,
    pub follow_distance: f32,
}

#[derive(Default, Component)]
#[storage(VecStorage)]
pub struct SearchForTarget {
    pub from_factions: Set<FactionId>,
    pub radius: f32,
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Faction {
    pub id: FactionId,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum FactionId {
    Pirates,
    Good,
}

///////////////
// Rendering //
///////////////

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Sprite {
    pub asset: SpriteAsset,
    pub size: Size2f,
}
pub type SpriteAsset = DirOrSingle<Arc<ImageAsset>>;

/////////////
// Utility //
/////////////

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Reflection {
    pub id: &'static str,
}

#[derive(Debug)]
pub enum DirOrSingle<T> {
    Single { value: T },
    Directional { north: T, east: T, south: T, west: T },
}

#[derive(Default, Debug, Component)]
#[storage(FlaggedStorage)]
pub struct Directional {
    pub direction: Direction,
}
