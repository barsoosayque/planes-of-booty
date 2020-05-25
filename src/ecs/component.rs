use crate::{
    assets::*,
    math::{Direction, Size2f, Vec2f},
};
use nphysics2d::{
    ncollide2d::shape::ShapeHandle,
    object::{DefaultBodyHandle, DefaultColliderHandle},
};
use specs::{Component, Entity, FlaggedStorage, VecStorage, World, WorldExt};
use std::{collections::HashSet as Set, fmt, sync::Arc};

/////////////////////////
// Inventory and Items //
/////////////////////////

pub type ItemBox = Option<Entity>;

#[derive(Default, Debug, Component)]
#[storage(VecStorage)]
pub struct Weaponry {
    pub primary: ItemBox,
    pub secondary: ItemBox,
}

#[derive(Default, Debug, Component)]
#[storage(VecStorage)]
pub struct Inventory {
    pub content: Content,
}
#[derive(Debug)]
pub struct Content(Vec<ItemBox>, Option<u32>);
impl Default for Content {
    fn default() -> Self { Content(vec![None], None) }
}
impl Content {
    pub fn add(&mut self, world: &World, item: Entity) {
        let (reflections, mut stacks) = (world.read_storage::<Reflection>(), world.write_storage::<Stackable>());
        let id = reflections.get(item).unwrap().id;
        let (mut current, size) = stacks.get(item).map(|s| (s.current, s.stack_size)).unwrap_or((1, 1));

        // Try to increment already existent same items
        if size > 1 {
            for e in self.0.iter_mut().filter_map(|i| i.as_mut()) {
                if reflections.get(*e).unwrap().id == id {
                    if let Some(mut e_stack) = stacks.get_mut(*e) {
                        let transfer_count = current.min(e_stack.stack_size - e_stack.current);
                        e_stack.current += transfer_count;
                        current = current.saturating_sub(transfer_count);
                        if current == 0 {
                            break;
                        }
                    }
                }
            }

            // We can guarantee that if stack size is > 1 then item
            // is Stackable
            stacks.get_mut(item).unwrap().current = current;
            if current == 0 {
                return;
            }
        }

        // Emplace new item in empty boxes
        for item_box in self.0.iter_mut() {
            if item_box.is_none() {
                item_box.replace(item);
                break;
            }
        }
    }

    pub fn maintain(&mut self) {
        // Add empty space if there is no space left
        if self.0.last().map(|x| x.is_some()).unwrap_or(false) {
            self.0.push(None);
        }
        
        if let Some(last_non_empty) = self.0.iter().rposition(|x| x.is_some()) {
            self.0.truncate(last_non_empty + 2);
        }
    }

    pub fn is_empty(&self) -> bool { self.0.is_empty() }

    pub fn iter(&self) -> impl Iterator<Item = &ItemBox> { self.0.iter() }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ItemBox> { self.0.iter_mut() }
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
impl fmt::Display for Rarity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Rarity::Common => "Common",
            Rarity::Rare => "Rare",
            Rarity::Epic => "Epic",
        })
    }
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Stackable {
    pub current: u32,
    pub stack_size: u32,
}
impl Default for Stackable {
    fn default() -> Self { Stackable { current: 1, stack_size: 1 } }
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Weapon {
    pub projectile_asset: Arc<ImageAsset>,
    pub projectile_velocity: f32,
    pub projectile_lifetime: f32,
    pub clip_size: u8,
    pub damage: u32,
    pub reloading_time: f32,
    pub accuracy: f32,
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
