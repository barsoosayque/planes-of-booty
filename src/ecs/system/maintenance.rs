use super::super::{component::*, resource::*, tag};
use crate::{math::*, ui::system::ImGuiSystem};
use ggez::input::{keyboard::KeyCode, mouse::MouseButton};
use specs::prelude::*;
use std::ops::DerefMut;

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

pub struct InputsSystem;
impl<'a> System<'a> for InputsSystem {
    type SystemData = (
        WriteStorage<'a, Movement>,
        ReadStorage<'a, Transform>,
        Read<'a, Inputs>,
        Read<'a, Camera>,
        ReadStorage<'a, tag::Player>,
        WriteStorage<'a, Weaponry>,
        WriteStorage<'a, WeaponProperties>,
    );

    fn run(
        &mut self,
        (mut movements, transforms, inputs, camera, tag, mut weaponries, mut wpn_props): Self::SystemData,
    ) {
        for (movement, _) in (&mut movements, &tag).join() {
            let mut direction = Vec2f::zero();
            if inputs.pressed_keys.contains(&KeyCode::W) {
                direction.y -= 1.0;
            };
            if inputs.pressed_keys.contains(&KeyCode::A) {
                direction.x -= 1.0;
            };
            if inputs.pressed_keys.contains(&KeyCode::S) {
                direction.y += 1.0;
            };
            if inputs.pressed_keys.contains(&KeyCode::D) {
                direction.x += 1.0;
            };
            movement.target_acceleration_normal = direction.try_normalize().unwrap_or_default();
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
                let (entity1, entity2) = (
                    physic_world.entity_for_collider(&handle1).unwrap(),
                    physic_world.entity_for_collider(&handle2).unwrap(),
                );

                let damage = Self::impact_factor(
                    &movements.get(*entity1).map(|m| m.velocity).unwrap_or(Vec2f::zero()),
                    &movements.get(*entity2).map(|m| m.velocity).unwrap_or(Vec2f::zero()),
                ) * Self::DAMAGE_MAX;

                if damage > 0.0 {
                    let damage_pack = (damage.floor() as u32, DamageType::Impact);
                    if let Some(rec) = dmg_recievers.get_mut(*entity1) {
                        rec.damage_queue.push(damage_pack);
                    }
                    if let Some(rec) = dmg_recievers.get_mut(*entity2) {
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
    type SystemData =
        (Entities<'a>, WriteExpect<'a, PhysicWorld>, ReadStorage<'a, tag::PendingDestruction>, ReadStorage<'a, Physic>);

    fn run(&mut self, (entities, mut pworld, to_destruct, physics): Self::SystemData) {
        for (e, physics_opt, _) in (&entities, (&physics).maybe(), &to_destruct).join() {
            if let Some(physic) = physics_opt {
                pworld.colliders.remove(physic.collide.0);
                pworld.bodies.remove(physic.body);
            }
            entities.delete(e).unwrap();
        }
    }
}

pub struct UiSystem<'a>(pub &'a mut ggez::Context, pub &'a mut ImGuiSystem);
impl<'s> System<'s> for UiSystem<'_> {
    type SystemData = (UiData<'s>, Read<'s, DeltaTime>, Write<'s, UiHub>);

    fn run(&mut self, (mut data, dt, mut hub): Self::SystemData) {
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
                log::debug!("Spawn {} using debug tools", id);
                let pos = data.camera.project(&data.inputs.mouse_pos);
                data.spawn_queue.0.push_back(SpawnItem::Entity(id.to_owned(), pos));
            } else if data.inputs.mouse_clicked.contains(&MouseButton::Right) {
                hub.debug_window.selected_entity = None;
            }
        }

        if data.inputs.mouse_clicked.contains(&MouseButton::Right) {
            hub.inventory_window.reset_dragging();
        }
    }
}
