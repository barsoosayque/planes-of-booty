use crate::math::Vec2f;
use crate::ecs::{component::*, tag};
use crate::assets::*;
use specs::{World, WorldExt, Entity, world::Builder};

pub fn player(world: &mut World, ctx: &mut ggez::Context, assets: &mut AssetManager) -> Entity {
        world
            .create_entity()
            .with(tag::Player)
            .with(Transform::default())
            .with(Movement { acc: Vec2f::new(10.0, 10.0), .. Movement::default() })
            .with(DirectionalSprite {
                north: assets.get::<ImageAsset>("/sprites/ship-north.png", ctx).unwrap(),
                east: assets.get::<ImageAsset>("/sprites/ship-east.png", ctx).unwrap(),
                south: assets.get::<ImageAsset>("/sprites/ship-south.png", ctx).unwrap(),
                west: assets.get::<ImageAsset>("/sprites/ship-west.png", ctx).unwrap(),
                width: 80.0, 
                height: 80.0
            })
            .build()
}
