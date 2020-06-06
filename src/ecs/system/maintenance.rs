use super::super::{component::*, resource::*, tag};
use crate::{arena, entity, math::*, read_event, ui::system::ImGuiSystem};
use ggez::input::{keyboard::KeyCode, mouse::MouseButton};
use itertools::Itertools;
use nphysics2d::{
    math::Isometry,
    ncollide2d::shape::{Cuboid, ShapeHandle},
    object::{BodyPartHandle, BodyStatus, ColliderDesc, RigidBodyDesc},
};
use rand::{distributions::uniform::Uniform, seq::SliceRandom, thread_rng, Rng};
use specs::prelude::*;
use std::ops::DerefMut;

pub struct ArenaSystem;
impl<'a> System<'a> for ArenaSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, tag::Player>,
        ReadStorage<'a, tag::LevelChanger>,
        WriteStorage<'a, tag::PendingDestruction>,
        WriteExpect<'a, PhysicWorld>,
        Write<'a, Arena>,
        Write<'a, SpawnQueue>,
    );

    fn run(
        &mut self,
        (entities, transforms, factions, player, lvl_changer, mut to_destruct, mut world, mut arena, mut spawn_queue): Self::SystemData,
    ) {
        // if no enemies left and no portal yet, spawn portal
        if !(&factions).join().any(|f| f.id != FactionId::Good) && (&lvl_changer).join().next().is_none() {
            spawn_queue.0.push_back(SpawnItem::Entity(entity::ID::Swirl, Point2f::zero(), vec![]));
        }

        if let Some(id) = arena.change_to.take() {
            // clear old entities
            for (e, _, _) in (&entities, &transforms, !&player).join() {
                to_destruct.insert(e, tag::PendingDestruction).unwrap();
            }
            arena::set(id, arena.deref_mut(), spawn_queue.deref_mut());
        }

        let size = arena.size;
        for (i, border) in arena.borders.iter_mut().enumerate() {
            let handle = match border {
                Some(collider) => collider,
                None => {
                    let body = world.bodies.insert(RigidBodyDesc::<f32>::new().status(BodyStatus::Static).build());
                    let shape = ShapeHandle::new(Cuboid::new([0.0, 0.0].into()));
                    let collider =
                        world.colliders.insert(ColliderDesc::<f32>::new(shape).build(BodyPartHandle(body, 0)));
                    border.replace(collider);
                    border.as_mut().unwrap()
                },
            };
            let collider = world.colliders.get_mut(*handle).unwrap();
            let d = if i < 2 { 1.0 } else { -1.0 };
            if i % 2 == 0 {
                collider.set_position(Isometry::translation(d * (size.width * 0.5 + 50.0), 0.0));
                collider.set_shape(ShapeHandle::new(Cuboid::new([50.0, size.height * 0.5].into())));
            } else {
                collider.set_position(Isometry::translation(0.0, d * (size.height * 0.5 + 50.0)));
                collider.set_shape(ShapeHandle::new(Cuboid::new([size.width * 0.5, 50.0].into())));
            }
        }
    }
}

pub struct CameraSystem;
impl<'a> System<'a> for CameraSystem {
    type SystemData = (WriteExpect<'a, Camera>, Read<'a, DeltaTime>, ReadStorage<'a, Transform>);

    fn run(&mut self, (mut camera, dt, transforms): Self::SystemData) {
        if let Some(target_transform) = camera.target.and_then(|e| transforms.get(e)) {
            let pos_dt = target_transform.pos - camera.pos;
            camera.pos += pos_dt * dt.0.as_secs_f32() * 4.0;
        }
    }
}

pub struct InventoryMaintenanceSystem;
impl<'a> System<'a> for InventoryMaintenanceSystem {
    type SystemData = WriteStorage<'a, Inventory>;

    fn run(&mut self, mut inventories: Self::SystemData) {
        for inv in (&mut inventories).join() {
            inv.content.maintain();
        }
    }
}

macro_rules! random_range {
    ($rng:expr; $from:expr => $to:expr) => {
        if let Some(range) = &$from {
            $to = $rng.sample(Uniform::new_inclusive(range.start(), range.end()));
        }
    };
}
#[derive(Default)]
pub struct RandomizedWeaponsSystem {
    reader_id: Option<ReaderId<ComponentEvent>>,
    inserted: BitSet,
}
impl<'a> System<'a> for RandomizedWeaponsSystem {
    type SystemData = (WriteStorage<'a, RandomizedWeaponProperties>, WriteStorage<'a, WeaponProperties>);

    fn run(&mut self, (mut randoms, mut props): Self::SystemData) {
        read_event!(randoms, self.reader_id.as_mut().unwrap(); Inserted => self.inserted);

        for (random, prop, _) in (&randoms, &mut props, &self.inserted).join() {
            let mut rng = thread_rng();
            random_range!(rng; random.clip_size => prop.clip_size);
            random_range!(rng; random.reloading_time => prop.reloading_time);
            random_range!(rng; random.cooldown_time => prop.cooldown_time);
            random_range!(rng; random.damage => prop.damage);
            random_range!(rng; random.accuracy => prop.accuracy);
        }

        randoms.clear();
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.reader_id = Some(world.write_storage::<RandomizedWeaponProperties>().register_reader());
    }
}
pub struct InteractionSystem;
impl<'a> System<'a> for InteractionSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, InteractionCache>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Inventory>,
        ReadStorage<'a, tag::Player>,
        ReadStorage<'a, tag::LevelChanger>,
    );

    fn run(&mut self, (entities, mut interaction, transforms, inventories, tag, lvl_changer): Self::SystemData) {
        for (transform, _) in (&transforms, &tag).join() {
            let near_inventory = (&entities, &inventories, &transforms, !&tag)
                .join()
                .map(|(e, _, t, _)| (e, (t.pos - transform.pos).length()))
                .filter(|(_, distance)| *distance <= 50.0)
                .fold1(|t1, t2| if t1.1 < t2.1 { t1 } else { t2 });
            interaction.near_inventory = near_inventory.map(|(e, _)| e);

            let near_level_changer = (&entities, &transforms, &lvl_changer)
                .join()
                .map(|(e, t, _)| (e, (t.pos - transform.pos).length()))
                .filter(|(_, distance)| *distance <= 50.0)
                .fold1(|t1, t2| if t1.1 < t2.1 { t1 } else { t2 });
            interaction.near_level_changer = near_level_changer.map(|(e, _)| e);
        }
    }
}
pub struct InputsSystem;
impl<'a> System<'a> for InputsSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Movement>,
        WriteExpect<'a, UiHub>,
        WriteExpect<'a, Arena>,
        Read<'a, InteractionCache>,
        Read<'a, Inputs>,
        Read<'a, Camera>,
        ReadStorage<'a, tag::Player>,
        WriteStorage<'a, tag::PendingDestruction>,
        WriteStorage<'a, Hotbar>,
        WriteStorage<'a, Consumer>,
        WriteStorage<'a, Weaponry>,
        WriteStorage<'a, WeaponProperties>,
        ReadStorage<'a, Consumable>,
        WriteStorage<'a, Stackable>,
    );

    fn run(
        &mut self,
        (
            entities,
            transforms,
            mut movements,
            mut ui,
            mut arena,
            interaction,
            inputs,
            camera,
            tag,
            mut to_destruct,
            mut hotbars,
            mut consumers,
            mut weaponries,
            mut wpn_props,
            consumables,
            mut stackables,
        ): Self::SystemData,
    ) {
        for (e, movement, _) in (&entities, &mut movements, &tag).join() {
            let mut direction = Vec2f::zero();
            for key in &inputs.pressed_keys {
                match key {
                    KeyCode::W => direction.y -= 1.0,
                    KeyCode::A => direction.x -= 1.0,
                    KeyCode::S => direction.y += 1.0,
                    KeyCode::D => direction.x += 1.0,
                    _ => (),
                }
            }
            movement.target_acceleration_normal = direction.try_normalize().unwrap_or_default();

            for key in &inputs.clicked_keys {
                match key {
                    KeyCode::Escape => {
                        ui.pause.is_opened = !ui.pause.is_opened;
                    },
                    KeyCode::I => {
                        ui.inventory_window.show_inventories_for.insert(e);
                    },
                    KeyCode::E => {
                        if let Some(near_inventory_e) = interaction.near_inventory {
                            ui.inventory_window.show_inventories_for.insert(near_inventory_e);
                        }
                        if interaction.near_level_changer.is_some() {
                            arena.difficulty *= 1.5;
                            let mut rng = thread_rng();
                            arena.change_to.replace(*arena::IDS.choose(&mut rng).unwrap());
                        }
                    },
                    KeyCode::Key1 | KeyCode::Key2 | KeyCode::Key3 | KeyCode::Key4 => {
                        let n = *key as usize - KeyCode::Key1 as usize;
                        if let (Some(consumer), Some(hotbar)) = (consumers.get_mut(e), hotbars.get_mut(e)) {
                            if let (Some(consumable), stackable) = (
                                hotbar.content[n].and_then(|i| consumables.get(i)),
                                hotbar.content[n].and_then(|i| stackables.get_mut(i)),
                            ) {
                                // check if we already have this consumable
                                if !consumer.handles.iter().any(|h| std::ptr::eq(h.behaviour, consumable.behaviour)) {
                                    let consume_item = if let Some(stackable) = stackable {
                                        if stackable.current > 1 {
                                            stackable.current -= 1;
                                            false
                                        } else {
                                            true
                                        }
                                    } else {
                                        true
                                    };
                                    if consume_item {
                                        to_destruct
                                            .insert(hotbar.content[n].take().unwrap(), tag::PendingDestruction)
                                            .unwrap();
                                    }
                                    consumer.handles.push(ConsumeHandle { behaviour: consumable.behaviour, time: 0.0 });
                                }
                            }
                        }
                    },
                    _ => (),
                }
            }
        }
        for (transform, weaponry, _) in (&transforms, &mut weaponries, &tag).join() {
            if let Some(props) = weaponry.primary.and_then(|i| wpn_props.get_mut(i)) {
                props.is_shooting = inputs.mouse_pressed.contains(&MouseButton::Left);
                props.shooting_normal = (camera.project(&inputs.mouse_pos).to_vector() - transform.pos).normalize()
            }

            if inputs.mouse_scroll != 0.0 && weaponry.primary.is_some() && weaponry.secondary.is_some() {
                std::mem::swap(&mut weaponry.primary, &mut weaponry.secondary);
            }
        }
    }
}

pub struct ImpactDamageSystem;
impl ImpactDamageSystem {
    const DAMAGE_MAX: f32 = 30.0;
    const VELOCITY_MAX: f32 = 400.0;
    const VELOCITY_MIN: f32 = 150.0;

    fn impact_factor(v1: &Vec2f, v2: &Vec2f) -> f32 {
        ((Vec2f::new(v1.x - v2.x, v1.y - v2.y).length() - Self::VELOCITY_MIN).max(0.0) / Self::VELOCITY_MAX).min(1.0)
    }
}
impl<'a> System<'a> for ImpactDamageSystem {
    type SystemData = (ReadExpect<'a, PhysicWorld>, ReadStorage<'a, Movement>, WriteStorage<'a, DamageReciever>);

    fn run(&mut self, (physic_world, movements, mut dmg_recievers): Self::SystemData) {
        use nphysics2d::ncollide2d::pipeline::narrow_phase::ContactEvent;
        for contact in physic_world.geometry_world.contact_events() {
            if let ContactEvent::Started(handle1, handle2) = contact {
                let (entity1, entity2) =
                    (physic_world.entity_for_collider(&handle1), physic_world.entity_for_collider(&handle2));

                let damage = Self::impact_factor(
                    &entity1.and_then(|e| movements.get(*e)).map(|m| m.velocity).unwrap_or(Vec2f::zero()),
                    &entity2.and_then(|e| movements.get(*e)).map(|m| m.velocity).unwrap_or(Vec2f::zero()),
                ) * Self::DAMAGE_MAX;

                if damage > 0.0 {
                    let damage_pack = (damage.floor() as u32, DamageType::Impact);
                    if let Some(rec) = entity1.and_then(|e| dmg_recievers.get_mut(*e)) {
                        rec.damage_queue.push(damage_pack);
                    }
                    if let Some(rec) = entity2.and_then(|e| dmg_recievers.get_mut(*e)) {
                        rec.damage_queue.push(damage_pack);
                    }
                }
            }
        }
    }
}

pub struct DamageSystem;
impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, DeltaTime>,
        WriteStorage<'a, HealthPool>,
        WriteStorage<'a, DamageReciever>,
        WriteStorage<'a, tag::PendingDestruction>,
    );

    fn run(&mut self, (entities, dt, mut hpools, mut dmg_recievers, mut to_destruct): Self::SystemData) {
        for (mut hpool, dmg_rec) in (&mut hpools.restrict_mut(), &mut dmg_recievers).join() {
            for (damage, damage_type) in dmg_rec.damage_queue.drain(..) {
                if dmg_rec.damage_immunity[damage_type].is_none() {
                    let hpool = hpool.get_mut_unchecked();
                    hpool.hp = hpool.hp.saturating_sub(damage);
                }
            }

            for (_, time_opt) in dmg_rec.damage_immunity.iter_mut() {
                if let Some(mut time) = time_opt.take() {
                    time -= dt.0.as_secs_f32();
                    if time > 0.0 {
                        time_opt.replace(time);
                    }
                }
            }
        }

        for (e, hpool) in (&entities, &hpools).join() {
            if hpool.hp <= 0 {
                to_destruct.insert(e, tag::PendingDestruction).unwrap();
            }
        }
    }
}

pub struct DestructionSystem;
impl<'a> System<'a> for DestructionSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, tag::PendingDestruction>,
        WriteStorage<'a, Target>,
        ReadStorage<'a, Physic>,
        WriteExpect<'a, PhysicWorld>,
    );

    fn run(&mut self, (entities, to_destruct, mut targets, physics, mut physic_world): Self::SystemData) {
        for (e, _) in (&entities, &to_destruct).join() {
            for t in (&mut targets).join() {
                if t.target == Some(e) {
                    t.target.take();
                }
            }

            if let Some(physic) = physics.get(e) {
                physic_world.colliders.remove(physic.colliders.real.0);
                if let Some((hitbox, _)) = physic.colliders.hitbox {
                    physic_world.colliders.remove(hitbox);
                }
                physic_world.bodies.remove(physic.body);
            }

            entities.delete(e).unwrap();
        }
    }
}

pub struct UiSystem<'a>(pub &'a mut ggez::Context, pub &'a mut ImGuiSystem);
impl<'s> System<'s> for UiSystem<'_> {
    type SystemData = (UiData<'s>, Read<'s, DeltaTime>, Write<'s, UiHub>, ReadStorage<'s, tag::Player>);

    fn run(&mut self, (mut data, dt, mut hub, tag): Self::SystemData) {
        hub.game_over.is_opened = (&tag).join().next().is_none();

        let (ctx, imgui) = (&mut self.0, &mut self.1);
        if imgui.update(ctx, dt.0, hub.deref_mut(), &mut data) {
            data.inputs.mouse_clicked.remove(&MouseButton::Left);
            data.inputs.mouse_pressed.remove(&MouseButton::Left);
        }

        if hub.menu.is_show_inventory {
            if let Some((player, _)) = (&data.entities, &data.player_tag).join().next() {
                hub.inventory_window.show_inventories_for.insert(player);
            }
            hub.menu.is_show_inventory = false;
        }

        if let Some(id) = hub.debug_window.selected_entity {
            if data.inputs.mouse_clicked.contains(&MouseButton::Left) {
                log::debug!("Spawn {:?} using debug tools", id);
                let pos = data.camera.project(&data.inputs.mouse_pos);
                data.spawn_queue.0.push_back(SpawnItem::Entity(id, pos, vec![]));
            } else if data.inputs.mouse_clicked.contains(&MouseButton::Right) {
                hub.debug_window.selected_entity = None;
            }
        }

        if data.inputs.mouse_clicked.contains(&MouseButton::Right) {
            hub.inventory_window.reset_dragging();
        }
    }
}
