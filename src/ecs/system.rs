use super::{component::*, resource::*, tag};
use crate::math::*;
use ggez::{graphics, input::keyboard::KeyCode, Context};
use specs::{Join, Read, ReadStorage, System, WriteStorage};

pub struct SpriteRenderSystem<'a>(pub &'a mut Context);
impl<'a> System<'a> for SpriteRenderSystem<'_> {
    type SystemData = (
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Movement>,
        ReadStorage<'a, DirectionalSprite>,
    );

    fn run(&mut self, (transforms, movements, sprites): Self::SystemData) {
        for (transform, movement, sprite) in (&transforms, &movements, &sprites).join() {
            let img = match Direction::from_vec2f(&movement.velocity) {
                Direction::North => &sprite.north.0,
                Direction::East => &sprite.east.0,
                Direction::South => &sprite.south.0,
                Direction::West => &sprite.west.0,
            };
            let scale = Vec2f::new(
                sprite.width / img.width() as f32,
                sprite.height / img.height() as f32,
            );

            let param = graphics::DrawParam::default().scale(scale).dest(
                (transform.pos - Vec2f::new(sprite.width * 0.5, sprite.height * 0.5)).to_point(),
            );
            graphics::draw(self.0, img, param).unwrap();
        }
    }
}

pub struct MovementSystem;
impl MovementSystem {
    const VELOCITY_DECREASE: f32 = 0.98;
}
impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Movement>,
        Read<'a, DeltaTime>,
    );

    fn run(&mut self, (mut transforms, mut movements, delta): Self::SystemData) {
        for (transform, movement) in (&mut transforms, &mut movements).join() {
            movement.acceleration =
                movement.target_acceleration_normal * movement.acceleration_flat;

            if movement.acceleration.equal(Vec2f::zero()).all() {
                movement.velocity *= Self::VELOCITY_DECREASE * (1.0 - delta.0.as_secs_f32());
            } else {
                let acc_change = lerp(
                    delta.0.as_secs_f32(),
                    1.0,
                    movement.acceleration_change_throttle,
                );
                movement.velocity = (movement.velocity + movement.acceleration * acc_change)
                    .with_max_length(movement.max_velocity);
            }
            transform.pos += movement.velocity * delta.0.as_secs_f32();
        }
    }
}

pub struct InputsSystem;
impl<'a> System<'a> for InputsSystem {
    type SystemData = (
        WriteStorage<'a, Movement>,
        Read<'a, Inputs>,
        WriteStorage<'a, tag::Player>,
    );

    fn run(&mut self, (mut movements, inputs, _): Self::SystemData) {
        for movement in (&mut movements).join() {
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
