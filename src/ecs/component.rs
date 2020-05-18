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
    pub acc: Vec2f,
    pub direction: Direction,
}

#[derive(Debug)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Default for Direction {
    fn default() -> Self {
        Self::North
    }
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
