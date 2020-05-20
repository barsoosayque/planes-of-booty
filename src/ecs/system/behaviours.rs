use super::super::{component::*, resource::*};
use crate::math::*;
use specs::{Join, Read, System, WriteStorage};

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
