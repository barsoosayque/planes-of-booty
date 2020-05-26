use super::super::{component::*, resource::*, tag};
use crate::{math::*, ui::system::ImGuiSystem};
use ggez::input::{keyboard::KeyCode, mouse::MouseButton};
use specs::prelude::*;
use std::ops::DerefMut;

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
        ReadStorage<'a, tag::Player>,
        ReadStorage<'a, Weaponry>,
        WriteStorage<'a, WeaponProperties>,
    );

    fn run(&mut self, (mut movements, transforms, inputs, tag, weaponries, mut wpn_props): Self::SystemData) {
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
                props.shooting_normal = (transform.pos - inputs.mouse_pos.to_vector()).normalize()
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
                data.spawn_queue.0.push_back(SpawnItem::Entity(id.to_owned(), data.inputs.mouse_pos));
            } else if data.inputs.mouse_clicked.contains(&MouseButton::Right) {
                hub.debug_window.selected_entity = None;
            }
        }

        if data.inputs.mouse_clicked.contains(&MouseButton::Right) {
            hub.inventory_window.reset_dragging();
        }
    }
}
