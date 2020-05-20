use crate::assets::*;
use crate::math::Vec2f;
use specs::{Component, VecStorage};
use std::sync::Arc;

#[derive(Default, Debug, Component)]
#[storage(VecStorage)]
pub struct Transform {
    pub pos: Vec2f,
    pub rotation: f32,
}

#[derive(Default, Debug, Component)]
#[storage(VecStorage)]
pub struct Movement {
    pub velocity: Vec2f,
    pub acceleration: Vec2f,
    pub target_acceleration_normal: Vec2f,

    pub max_velocity: f32,
    pub acceleration_flat: f32,
    pub acceleration_change_throttle: f32,
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct DirectionalSprite {
    pub north: Arc<ImageAsset>,
    pub east: Arc<ImageAsset>,
    pub south: Arc<ImageAsset>,
    pub west: Arc<ImageAsset>,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Sprite {
    pub asset: Arc<ImageAsset>,
    pub width: f32,
    pub height: f32,
}
