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
        ReadStorage<'a, Weaponry>,
        WriteStorage<'a, WeaponProperties>,
    );

    fn run(&mut self, (mut movements, transforms, inputs, camera, tag, weaponries, mut wpn_props): Self::SystemData) {
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
        for (transform, weaponry, _) in (&transforms, &weaponries, &tag).join() {
            if let Some(props) = weaponry.primary.and_then(|i| wpn_props.get_mut(i)) {
                props.is_shooting = inputs.mouse_pressed.contains(&MouseButton::Left);
                props.shooting_normal = (camera.project(&inputs.mouse_pos).to_vector() - transform.pos).normalize()
            }
        }
    }
}

pub struct WatchDeadSystem;
impl<'a> System<'a> for WatchDeadSystem {
    type SystemData = (Entities<'a>, ReadStorage<'a, HealthPool>);

    fn run(&mut self, (entities, hpools): Self::SystemData) {
        for (e, hpool) in (&entities, &hpools).join() {
            if hpool.hp <= 0 {
                entities.delete(e).unwrap();
            }
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
