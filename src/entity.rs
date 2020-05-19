use crate::assets::*;
use crate::ecs::{component::*, tag};
use specs::{world::Builder, Entity, World, WorldExt};

pub fn player(world: &mut World, ctx: &mut ggez::Context, assets: &mut AssetManager) -> Entity {
    world
        .create_entity()
        .with(tag::Player)
        .with(Transform::default())
        .with(Movement {
            max_velocity: 200.0,
            acceleration_flat: 150.0,
            acceleration_change_throttle: 0.02,
            ..Movement::default()
        })
        .with(DirectionalSprite {
            north: assets
                .get::<ImageAsset>("/sprites/ship-north.png", ctx)
                .unwrap(),
            east: assets
                .get::<ImageAsset>("/sprites/ship-east.png", ctx)
                .unwrap(),
            south: assets
                .get::<ImageAsset>("/sprites/ship-south.png", ctx)
                .unwrap(),
            west: assets
                .get::<ImageAsset>("/sprites/ship-west.png", ctx)
                .unwrap(),
            width: 100.0,
            height: 100.0,
        })
        .build()
}
