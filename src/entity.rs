#![allow(warnings)]
use crate::{
    assets::*,
    ecs::{
        component,
        component::{ShapeshifterData, ShapeshifterForm},
        resource, tag,
    },
    math::*,
};
use nphysics2d::ncollide2d::pipeline::object::CollisionGroups;
use specs::{Entity, LazyUpdate, World, WorldExt};

// see `build/build.rs` for code generation
include!(concat!(env!("OUT_DIR"), "/generated/entity.rs"));

struct CrabUnderwaterForm;
impl ShapeshifterForm for CrabUnderwaterForm {
    fn time(&self) -> f32 { 4.0 }

    fn can_update(&self, e: Entity, world: &World) -> bool {
        world.read_storage::<component::Target>().get(e).map(|t| t.target.is_some()).unwrap_or(true)
    }

    fn on_begin(&self, e: Entity, update: &LazyUpdate, (ctx, assets): ShapeshifterData) {
        let new_asset = assets.get::<ImageAsset>("/sprites/entity/crab-underwater.png", ctx).unwrap();
        update.insert(e, component::Sprite {
            asset: component::SpriteAsset::Single { value: new_asset },
            size: Size2f::new(40.0, 80.0),
        });
        update.insert(e, component::FollowTarget { keep_distance: 50.0, follow_distance: 500.0 });
        update.exec(move |world| {
            let physics = world.read_storage::<component::Physic>();
            let mut physic_world = world.write_resource::<resource::PhysicWorld>();
            let physic = physics.get(e).unwrap();
            let collide = physic_world.colliders.get_mut(physic.collide.0).unwrap();
            collide.set_collision_groups(collide.collision_groups().clone().with_blacklist(&component::NO_COLLISION));
        });
    }

    fn on_end(&self, e: Entity, update: &LazyUpdate, (ctx, assets): ShapeshifterData) {
        update.remove::<component::FollowTarget>(e);
        update.exec(move |world| {
            let physics = world.read_storage::<component::Physic>();
            let mut physic_world = world.write_resource::<resource::PhysicWorld>();
            let physic = physics.get(e).unwrap();
            let collide = physic_world.colliders.get_mut(physic.collide.0).unwrap();
            collide.set_collision_groups(collide.collision_groups().clone().with_blacklist(&[]));
        });
    }
}

struct CrabShooterForm;
impl ShapeshifterForm for CrabShooterForm {
    fn time(&self) -> f32 { 6.0 }

    fn on_begin(&self, e: Entity, update: &LazyUpdate, (ctx, assets): ShapeshifterData) {
        let new_asset = assets.get::<ImageAsset>("/sprites/entity/crab-shoot.png", ctx).unwrap();
        update.insert(e, component::Sprite {
            asset: component::SpriteAsset::Single { value: new_asset },
            size: Size2f::new(120.0, 80.0),
        });
        update.insert(e, component::ShootTarget { radius: 400.0 });
    }

    fn on_end(&self, e: Entity, update: &LazyUpdate, (ctx, assets): ShapeshifterData) {
        update.remove::<component::ShootTarget>(e);
    }
}

struct CrabJrUnderwaterForm;
impl ShapeshifterForm for CrabJrUnderwaterForm {
    fn time(&self) -> f32 { 3.0 }

    fn can_update(&self, e: Entity, world: &World) -> bool {
        world.read_storage::<component::Target>().get(e).map(|t| t.target.is_some()).unwrap_or(true)
    }

    fn on_begin(&self, e: Entity, update: &LazyUpdate, (ctx, assets): ShapeshifterData) {
        let new_asset = assets.get::<ImageAsset>("/sprites/entity/crab-jr-underwater.png", ctx).unwrap();
        update.insert(e, component::Sprite {
            asset: component::SpriteAsset::Single { value: new_asset },
            size: Size2f::new(100.0, 40.0),
        });
        update.insert(e, component::FollowTarget { keep_distance: 100.0, follow_distance: 400.0 });
    }
}

struct CrabJrShooterForm;
impl ShapeshifterForm for CrabJrShooterForm {
    fn time(&self) -> f32 { 1.0 }

    fn can_update(&self, e: Entity, world: &World) -> bool {
        world.read_storage::<component::Target>().get(e).map(|t| t.target.is_some()).unwrap_or(true)
    }

    fn on_begin(&self, e: Entity, update: &LazyUpdate, (ctx, assets): ShapeshifterData) {
        let new_asset = assets.get::<ImageAsset>("/sprites/entity/crab-jr-shoot.png", ctx).unwrap();
        update.insert(e, component::Sprite {
            asset: component::SpriteAsset::Single { value: new_asset },
            size: Size2f::new(100.0, 40.0),
        });
        update.remove::<component::FollowTarget>(e);
    }
}

struct CrabJrShockedForm;
impl ShapeshifterForm for CrabJrShockedForm {
    fn time(&self) -> f32 { 4.0 }

    fn on_begin(&self, e: Entity, update: &LazyUpdate, (ctx, assets): ShapeshifterData) {
        let new_asset = assets.get::<ImageAsset>("/sprites/entity/crab-jr-shock.png", ctx).unwrap();
        update.insert(e, component::Sprite {
            asset: component::SpriteAsset::Single { value: new_asset },
            size: Size2f::new(100.0, 40.0),
        });
        update.insert(e, component::ShootTarget { radius: 400.0 });
    }

    fn on_end(&self, e: Entity, update: &LazyUpdate, (ctx, assets): ShapeshifterData) {
        update.remove::<component::ShootTarget>(e);
    }
}
