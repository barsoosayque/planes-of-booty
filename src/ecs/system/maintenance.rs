use super::super::{component::*, resource::*, tag};
use crate::{math::*, ui::system::ImGuiSystem};
use ggez::input::{keyboard::KeyCode, mouse::MouseButton};
use specs::{Join, Read, System, Write, WriteStorage};

pub struct InputsSystem;
impl<'a> System<'a> for InputsSystem {
    type SystemData = (WriteStorage<'a, Movement>, Read<'a, Inputs>, WriteStorage<'a, tag::Player>);

    fn run(&mut self, (mut movements, inputs, tag): Self::SystemData) {
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
    }
}

pub struct UiSystem<'a>(pub &'a mut ggez::Context, pub &'a mut ImGuiSystem);
impl<'a> System<'a> for UiSystem<'_> {
    type SystemData = (Write<'a, SpawnQueue>, Write<'a, UiHub>, Write<'a, Inputs>, Read<'a, DeltaTime>);

    fn run(&mut self, (mut spawn_queue, mut ui_hub, mut inputs, delta): Self::SystemData) {
        use std::ops::DerefMut;
        let consume = self.1.update(self.0, ui_hub.deref_mut(), delta.0);
        if consume {
            inputs.mouse_clicked.remove(&MouseButton::Left);
        }

        if let Some(id) = ui_hub.debug_window.selected_entity {
            if inputs.mouse_clicked.contains(&MouseButton::Left) {
                log::debug!("Spawn {} using debug tools", id);
                spawn_queue.0.push_back(SpawnItem { id: id.to_owned(), pos: inputs.mouse_pos });
            } else if inputs.mouse_clicked.contains(&MouseButton::Right) {
                ui_hub.debug_window.selected_entity = None;
            }
        }
    }
}
