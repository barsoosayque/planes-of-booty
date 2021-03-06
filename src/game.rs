use crate::{
    assets::*,
    ecs::{component::*, resource::*, system::*, tag},
    entity, item,
    main_menu::MainMenu,
    math::*,
    particle,
    scene::{Scene, SceneCommand},
    ui::ImGuiSystem,
};
use ggez::{
    event::EventHandler,
    graphics,
    input::keyboard::{KeyCode, KeyMods},
    timer, Context, GameResult,
};
use itertools::Itertools;
use nphysics2d::{
    math::{Isometry, Velocity},
    ncollide2d::{pipeline::object::CollisionGroups, shape},
    object::{BodyPartHandle, BodyStatus, ColliderDesc, RigidBodyDesc},
};
use specs::prelude::*;

pub struct Game {
    world: World,
    dispatcher: Dispatcher<'static, 'static>,
    imgui: ImGuiSystem,
}

impl Game {
    fn prespawn(&mut self, ctx: &mut Context) {
        let player = entity::spawn_player(&self.world, ctx, &mut self.world.write_resource::<AssetManager>());
        self.world.write_resource::<Camera>().target = Some(player);
    }

    pub fn new(ctx: &mut Context) -> Self {
        let imgui = ImGuiSystem::new(ctx);
        let mut world = World::new();
        let mut dispatcher = DispatcherBuilder::new()
            .with(ArenaSystem, "arena_system", &[])
            .with(ConsumablesSystem, "consumables_system", &[])
            .with(InteractionSystem, "interaction_system", &[])
            .with(CameraSystem, "camera_system", &[])
            .with(ParticlesSystem, "particles_system", &[])
            .with(SpriteDamageBlinkSystem::default(), "sprite_damage_blink_system", &[])
            .with(SearchForTargetSystem, "search_for_target_system", &[])
            .with(FollowTargetSystem::default(), "follow_target_system", &["search_for_target_system"])
            .with(ShootTargetSystem::default(), "shoot_target_system", &["search_for_target_system"])
            .with(InputsSystem, "inputs_system", &[])
            .with(DirectionalSystem, "directional_system", &[])
            .with(DirectionalCollidersSystem::default(), "directional_colliders_system", &["directional_system"])
            .with(PhysicTransformSyncSystem::default(), "physic_transform_sync_system", &[])
            .with(PhysicSystem, "physic_system", &["directional_colliders_system", "physic_transform_sync_system"])
            .with(DistanceCounterSystem, "distance_counter_system", &["physic_system"])
            .with(ContainerSinkSystem, "container_sink_system", &[])
            .with(InventoryMaintenanceSystem, "inv_maintenance_system", &[])
            .with(RandomizedWeaponsSystem::default(), "randomized_weapons_system", &[])
            .with(ProjectileSystem, "projectile_system", &["physic_system"])
            .with(ImpactDamageSystem, "impact_damage_system", &["physic_system"])
            .with(ShotsDodgerSystem, "shots_dodger_system", &["projectile_system", "impact_damage_system"])
            .with(DamageSystem, "damage_system", &["shots_dodger_system", "projectile_system", "impact_damage_system"])
            .with(WeaponrySystem, "weaponry_system", &["inputs_system", "damage_system"])
            .with(DistanceLimitingSystem, "distance_limiting_system", &["distance_counter_system"])
            // barrier for "on destruction" systems
            .with_barrier()
            .with(ExplodeOnDeathSystem, "explode_on_death_system", &[])
            .with(LootGenerateSystem, "loot_generate_system", &[])
            // Force destruction system to run the last
            .with_thread_local(DestructionSystem)
            .build();
        world.insert(DeltaTime(std::time::Duration::new(0, 0)));
        world.insert(Camera::default());
        world.insert(UiHub::default());
        world.insert(SpawnQueue::default());
        world.insert(AssetManager::default());
        world.insert(SceneControls { is_debug: std::env::args().any(|a| a == "--debug"), ..SceneControls::default() });
        world.insert(Arena::default());
        world.insert(PhysicWorld::new(Vec2f::new(0.0, 0.0)));
        world.register::<tag::Player>();
        world.register::<tag::LevelChanger>();
        world.register::<AvoidShots>();
        world.register::<Reflection>();
        world.register::<Shapeshifter>();
        world.register::<DistanceCounter>();
        world.register::<DistanceLimited>();
        world.register::<Movement>();
        world.register::<Transform>();
        world.register::<Sprite>();
        world.register::<SpriteBlink>();
        world.register::<Target>();
        world.register::<SearchForTarget>();
        world.register::<FollowTarget>();
        world.register::<ShootTarget>();
        world.register::<Faction>();
        world.register::<Physic>();
        world.register::<Directional>();
        world.register::<Inventory>();
        world.register::<Named>();
        world.register::<Quality>();
        world.register::<Stackable>();
        world.register::<RandomizedWeaponProperties>();
        world.register::<WeaponProperties>();
        world.register::<WeaponAttack>();
        world.register::<Weaponry>();
        world.register::<Hotbar>();
        world.register::<HealthPool>();
        world.register::<DamageDealer>();
        world.register::<DamageReciever>();
        world.register::<Projectile>();
        world.register::<SharedParticleDef>();
        world.register::<ParticleProperties>();
        world.register::<Consumable>();
        world.register::<Consumer>();
        dispatcher.setup(&mut world);

        let mut game = Self { world, dispatcher, imgui };
        game.prespawn(ctx);
        game
    }
}

impl Scene for Game {
    fn next_command(&self) -> Option<SceneCommand> {
        let scene_controls = self.world.read_resource::<SceneControls>();
        if scene_controls.queue_exit {
            Some(SceneCommand::ReplaceAll(|ctx| Box::new(MainMenu::new(ctx))))
        } else if scene_controls.queue_restart {
            Some(SceneCommand::ReplaceAll(|ctx| Box::new(Self::new(ctx))))
        } else {
            None
        }
    }

    fn draw_prev(&self) -> bool { false }
}
impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // TODO: Move all of this to some system
        {
            // update delta time
            let mut delta = self.world.write_resource::<DeltaTime>();
            delta.0 = timer::delta(ctx);
        };

        {
            // update inputs
            use ggez::input::{keyboard, mouse};
            let mut inputs = self.world.write_resource::<Inputs>();
            inputs.clicked_keys = inputs.pressed_keys.difference(&keyboard::pressed_keys(ctx)).copied().collect();
            inputs.pressed_keys = keyboard::pressed_keys(ctx).to_owned();
            inputs.mouse_pos = Point2f::from(mouse::position(ctx));
            let new_press: std::collections::HashSet<mouse::MouseButton> =
                [mouse::MouseButton::Left, mouse::MouseButton::Right]
                    .iter()
                    .cloned()
                    .filter(|btn| mouse::button_pressed(ctx, *btn))
                    .collect();
            inputs.mouse_clicked = inputs.mouse_pressed.difference(&new_press).copied().collect();
            inputs.mouse_pressed = new_press;
        }

        // Systems can spawn new stuff using SpawnQueue resource
        // TODO: make this LazyUpdate system
        for item in self.world.write_resource::<SpawnQueue>().0.drain(..) {
            let mut assets = self.world.write_resource::<AssetManager>();
            match item {
                SpawnItem::Entity(id, pos, items) => {
                    let e = entity::spawn(id, &self.world, ctx, &mut assets);
                    if let Some(transform) = self.world.write_storage::<Transform>().get_mut(e) {
                        transform.pos = pos.to_vector();
                    }
                    // can't create entities while any storage is borrowed
                    if self.world.read_storage::<Inventory>().contains(e) {
                        let items =
                            items.into_iter().map(|id| item::spawn(id, &self.world, ctx, &mut assets)).collect_vec();
                        if let Some(inventory) = self.world.write_storage::<Inventory>().get_mut(e) {
                            for item in items {
                                inventory.content.add(&self.world, item);
                            }
                        }
                    }
                },
                SpawnItem::Particle(id, pos) => {
                    let e = particle::spawn(id, &self.world, ctx, &mut assets);
                    if let Some(transform) = self.world.write_storage::<Transform>().get_mut(e) {
                        transform.pos = pos.to_vector();
                    }
                },
                SpawnItem::Item(id, count, to_e) => {
                    let e = item::spawn(id, &self.world, ctx, &mut assets);
                    if let Some(stack) = self.world.write_storage::<Stackable>().get_mut(e) {
                        stack.current = count;
                    }
                    if let Some(inventory) = self.world.write_storage::<Inventory>().get_mut(to_e) {
                        inventory.content.add(&self.world, e);
                    }
                },
                SpawnItem::Projectile(def) => {
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
                    let collider = phys_world.colliders.insert(
                        ColliderDesc::new(shape.clone())
                            .sensor(true)
                            .collision_groups(
                                CollisionGroups::new()
                                    .with_membership(&[CollisionGroup::Projectiles as usize])
                                    .with_blacklist(
                                        &def.ignore_groups.iter().cloned().map(|g| g as usize).collect::<Vec<usize>>(),
                                    ),
                            )
                            .build(BodyPartHandle(body, 0)),
                    );
                    let mut builder = self.world.create_entity_unchecked();
                    if let Some(asset) = &def.asset {
                        builder = builder.with(Sprite {
                            asset: SpriteAsset::Single { value: assets.get::<ImageAsset>(&asset, ctx).unwrap() },
                            size: def.size,
                        })
                    }
                    let entity = builder
                        .with(Transform {
                            pos: def.pos.to_vector(),
                            rotation: if def.rotate_projectile {
                                def.velocity.angle_from_x_axis()
                            } else {
                                Angle2f::zero()
                            },
                            ..Transform::default()
                        })
                        .with(DistanceLimited { limit: def.distance })
                        .with(DistanceCounter::default())
                        .with(DamageDealer { damage: def.damage.0, damage_type: def.damage.1 })
                        .with(Physic {
                            body: body,
                            colliders: PhysicColliders {
                                real: (collider, CollideShapeHandle::Single { value: shape.clone() }),
                                hitbox: None,
                            },
                        })
                        .with(Projectile { def: def })
                        .build();
                    phys_world.bodies.rigid_body_mut(body).unwrap().set_user_data(Some(Box::new(entity)));
                    phys_world.colliders.get_mut(collider).unwrap().set_user_data(Some(Box::new(entity)));
                },
            }
        }

        self.world.maintain();

        // run ui system before any other system so it can
        // consume input events
        UiSystem(ctx, &mut self.imgui).run_now(&self.world);
        if self.world.read_resource::<UiHub>().pause.is_opened {
            return Ok(());
        }

        self.dispatcher.dispatch(&self.world);
        // shapeshifter is a special kind of system, as it requires
        // ggez context
        ShapeshifterSystem(ctx).run_now(&self.world);

        // reset inputs
        self.world.write_resource::<Inputs>().mouse_scroll = 0.0;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let scene_controls = self.world.read_resource::<SceneControls>();
        self.world.write_resource::<Camera>().apply(ctx);
        graphics::clear(ctx, graphics::Color::from_rgb_u32(0x7cd6d4));
        MapRenderingSystem(ctx).run_now(&self.world);
        if scene_controls.is_debug_info {
            DebugInfoRenderSystem(ctx).run_now(&self.world);
        }
        if scene_controls.is_debug_targeting {
            DebugTargetRenderSystem(ctx).run_now(&self.world);
        }
        ParticleRenderSystem(ctx).run_now(&self.world);
        SpriteRenderSystem(ctx).run_now(&self.world);
        if scene_controls.is_debug_physic {
            DebugPhysicRenderSystem(ctx).run_now(&self.world);
        }
        self.world.write_resource::<Camera>().revert(ctx);
        UiRenderSystem(ctx, &mut self.imgui).run_now(&self.world);
        graphics::present(ctx)
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        self.world.write_resource::<Inputs>().mouse_scroll = y;
    }

    fn key_down_event(&mut self, _: &mut Context, _: KeyCode, _: KeyMods, _: bool) {}

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, width, height)).unwrap();
    }
}
