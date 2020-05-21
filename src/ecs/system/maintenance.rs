use super::super::{component::*, resource::*, tag};
use crate::math::*;
use crate::ui::system::ImGuiSystem;
use ggez::input::keyboard::KeyCode;
use specs::{Join, Read, System, Write, WriteStorage};

pub struct InputsSystem;
impl<'a> System<'a> for InputsSystem {
    type SystemData = (
        WriteStorage<'a, Movement>,
        Read<'a, Inputs>,
        WriteStorage<'a, tag::Player>,
    );

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

pub struct GameUiSystem<'a>(pub &'a mut ggez::Context, pub &'a mut ImGuiSystem);
impl<'a> System<'a> for GameUiSystem<'_> {
    type SystemData = Write<'a, UiHub>;

    fn run(&mut self, mut ui_hub: Self::SystemData) {
        use std::ops::DerefMut;
        self.1.render(self.0, ui_hub.deref_mut());
    }
}
