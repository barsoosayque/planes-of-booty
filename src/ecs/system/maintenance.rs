use super::super::{component::*, resource::*, tag};
use crate::math::*;
use ggez::input::keyboard::KeyCode;
use specs::{Join, Read, System, WriteStorage};

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
