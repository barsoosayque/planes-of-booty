use crate::{
    assets::*,
    math::{Direction, Size2f, Vec2f},
};
use nphysics2d::{
    ncollide2d::shape::ShapeHandle,
    object::{DefaultBodyHandle, DefaultColliderHandle},
};
use specs::{Component, Entity, FlaggedStorage, VecStorage};
use std::{collections::HashSet as Set, sync::Arc};

/////////////////////////
// Inventory and Items //
/////////////////////////

#[derive(Default, Debug, Component)]
#[storage(VecStorage)]
pub struct Inventory {
    pub content: Vec<ItemStack>,
}
#[derive(Debug)]
pub struct ItemStack {
    pub item: Entity,
    pub size: u32
}

#[derive(Default, Debug, Component)]
#[storage(VecStorage)]
pub struct Named {
    pub name: &'static str,
    pub description: &'static str
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Quality {
    pub rarity: Rarity,
}
#[derive(Debug)]
pub enum Rarity {
    Common, Rare, Epic
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
