use crate::{
    assets::*,
    ecs::{component::*, resource::*, system::*, tag},
    entity, item,
    math::*,
    ui::ImGuiSystem,
};
use ggez::{event::EventHandler, graphics, timer, Context, GameResult};
use nphysics2d::{
    math::{Isometry, Velocity},
    ncollide2d::shape,
    object::{BodyPartHandle, BodyStatus, ColliderDesc, RigidBodyDesc},
};
use specs::prelude::*;

pub struct Game {
    world: World,
    dispatcher: Dispatcher<'static, 'static>,
    imgui: ImGuiSystem,
}

impl Game {
    fn prespawn(&mut self) {
        let queue = &mut self.world.write_resource::<SpawnQueue>().0;
        queue.push_back(SpawnItem::Entity("player".into(), Point2f::new(300.0, 300.0)));
    }

    pub fn new(ctx: &mut Context) -> Self {
        let imgui = ImGuiSystem::new(ctx);
        let mut world = World::new();
        let mut dispatcher = DispatcherBuilder::new()
            .with(WatchDeadSystem, "watch_dead_system", &[])
            .with(SearchForTargetSystem, "search_for_target_system", &[])
            .with(FollowTargetSystem, "follow_target_system", &[])
            .with(InputsSystem, "inputs_system", &[])
            .with(DirectionalSystem, "directional_system", &[])
            .with(DirectionalCollidersSystem::default(), "directional_colliders_system", &["directional_system"])
            .with(PhysicTransformSyncSystem::default(), "physic_transform_sync_system", &[])
            .with(PhysicSystem, "physic_system", &["directional_colliders_system", "physic_transform_sync_system"])
            .with(DistanceCounterSystem, "distance_counter_system", &["physic_system"])
            .with(DistanceLimitingSystem, "distance_limiting_system", &["distance_counter_system"])
            .with(InventoryMaintenanceSystem, "inv_maintenance_system", &[])
            .with(WeaponrySystem, "weaponry_system", &["inputs_system"])
            .with(ProjectileSystem, "projectile_system", &["physic_system"])
            .build();
        world.insert(DeltaTime(std::time::Duration::new(0, 0)));
        world.insert(UiHub::default());
        world.insert(SpawnQueue::default());
        world.insert(AssetManager::default());
        world.insert(Settings::default());
        world.insert(PhysicWorld::new(Vec2f::new(0.0, 0.0)));
        world.register::<tag::Player>();
        world.register::<Reflection>();
        world.register::<DistanceCounter>();
        world.register::<DistanceLimited>();
        world.register::<Movement>();
        world.register::<Transform>();
        world.register::<Sprite>();
        world.register::<Target>();
        world.register::<SearchForTarget>();
        world.register::<FollowTarget>();
        world.register::<Faction>();
        world.register::<Physic>();
        world.register::<Directional>();
        world.register::<Inventory>();
        world.register::<Named>();
        world.register::<Quality>();
        world.register::<Stackable>();
        world.register::<WeaponProperties>();
        world.register::<WeaponAttack>();
        world.register::<Weaponry>();
        world.register::<HealthPool>();
        world.register::<DamageDealer>();
        dispatcher.setup(&mut world);

        let mut game = Self { world, dispatcher, imgui };
        game.prespawn();
        game
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        {
            // update delta time
            let mut delta = self.world.write_resource::<DeltaTime>();
            delta.0 = timer::delta(ctx);
        };

        {
            // update inputs
            use ggez::input::{keyboard, mouse};
            let mut inputs = self.world.write_resource::<Inputs>();
            inputs.pressed_keys = keyboard::pressed_keys(ctx).to_owned();
            inputs.mouse_pos = Point2f::from(mouse::position(ctx));
            let new_press: std::collections::HashSet<mouse::MouseButton> =
                [mouse::MouseButton::Left, mouse::MouseButton::Right]
                    .iter()
                    .cloned()
                    .filter(|btn| mouse::button_pressed(ctx, *btn))
                    .collect();
            inputs.mouse_clicked = inputs.mouse_pressed.difference(&new_press).cloned().collect();
            inputs.mouse_pressed = new_press;
        }

        // run ui system before any other system so it can
        // consume input events
        UiSystem(ctx, &mut self.imgui).run_now(&self.world);
        self.dispatcher.dispatch(&self.world);

        // Systems can spawn new stuff using SpawnQueue resource
        for item in self.world.write_resource::<SpawnQueue>().0.drain(..) {
            match item {
                SpawnItem::Entity(id, pos) => {
                    let e = entity::spawn(&id, &self.world, ctx);
                    if let Some(transform) = self.world.write_storage::<Transform>().get_mut(e) {
                        transform.pos = pos.to_vector();
                    }
                },
                SpawnItem::Item(id, count, to_e) => {
                    let e = item::spawn(&id, &self.world, ctx);
                    if let Some(stack) = self.world.write_storage::<Stackable>().get_mut(e) {
                        stack.current = count;
                    }
                    if let Some(inventory) = self.world.write_storage::<Inventory>().get_mut(to_e) {
                        inventory.content.add(&self.world, e);
                    }
                },
                SpawnItem::Projectile(def) => {
                    let mut assets = self.world.write_resource::<AssetManager>();
                    let mut phys_world = self.world.write_resource::<PhysicWorld>();
                    let body = phys_world.bodies.insert(
                        RigidBodyDesc::new()
                            .status(BodyStatus::Kinematic)
                            .position(Isometry::translation(def.pos.x, def.pos.y))
                            .velocity(Velocity::linear(def.velocity.x, def.velocity.y))
                            .build(),
                    );
                    let shape = shape::ShapeHandle::new(shape::Cuboid::new(
                        [def.size.width * 0.5, def.size.height * 0.5].into(),
                    ));
                    let collider = phys_world
                        .colliders
                        .insert(ColliderDesc::new(shape.clone()).sensor(true).build(BodyPartHandle(body, 0)));
                    let entity = self
                        .world
                        .create_entity_unchecked()
                        .with(Transform::default())
                        .with(DistanceLimited { limit: def.distance })
                        .with(DistanceCounter::default())
                        .with(DamageDealer { damage: def.damage })
                        .with(Physic {
                            body: body,
                            collide: (collider, CollideShapeHandle::Single { value: shape.clone() }),
                        })
                        .with(Sprite {
                            asset: SpriteAsset::Single { value: assets.get::<ImageAsset>(&def.asset, ctx).unwrap() },
                            size: def.size,
                        })
                        .build();
                    phys_world.bodies.rigid_body_mut(body).unwrap().set_user_data(Some(Box::new(entity)));
                    phys_world.colliders.get_mut(collider).unwrap().set_user_data(Some(Box::new(entity)));
                },
            }
        }

        self.world.maintain();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let settings = self.world.read_resource::<Settings>();
        graphics::clear(ctx, graphics::Color::from_rgb_u32(0x7cd6d4));
        if settings.is_debug_info {
            DebugInfoRenderSystem(ctx).run_now(&self.world);
        }
        if settings.is_debug_targeting {
            DebugTargetRenderSystem(ctx).run_now(&self.world);
        }
        SpriteRenderSystem(ctx).run_now(&self.world);
        if settings.is_debug_physic {
            DebugPhysicRenderSystem(ctx).run_now(&self.world);
        }
        UiRenderSystem(ctx, &mut self.imgui).run_now(&self.world);
        graphics::present(ctx)
    }
}
