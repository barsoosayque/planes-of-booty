use super::super::{component::*, resource::*};
use crate::math::*;
use log::debug;
use specs::{Entities, Join, Read, ReadStorage, System, WriteStorage};

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

pub struct SearchForTargetSystem;
impl<'a> System<'a> for SearchForTargetSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Target>,
        ReadStorage<'a, SearchForTarget>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, Transform>,
    );

    fn run(&mut self, (entities, mut targets, searches, factions, transforms): Self::SystemData) {
        for (e_target, transform, faction) in (&entities, &transforms, &factions).join() {
            for (e, target, search, search_transform) in
                (&entities, &mut targets, &searches, &transforms).join()
            {
                if target.target.is_some() {
                    continue;
                }

                let area = Circle2f::new(search_transform.pos.to_point(), search.radius);
                if search.from_factions.contains(&faction.id)
                    && area.contains(transform.pos.to_point())
                {
                    debug!("{:?} found new target {:?}", e, e_target);
                    target.target = Some(e_target)
                }
            }
        }
    }
}

pub struct FollowTargetSystem;
impl<'a> System<'a> for FollowTargetSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, FollowTarget>,
        WriteStorage<'a, Target>,
        WriteStorage<'a, Movement>,
        ReadStorage<'a, Transform>,
    );

    fn run(
        &mut self,
        (entities, follows, mut targets, mut movements, transforms): Self::SystemData,
    ) {
        for (e, follow, target, movement, transform) in (
            &entities,
            &follows,
            &mut targets,
            &mut movements,
            &transforms,
        )
            .join()
        {
            if let Some(target_transform) = target.target.and_then(|e| transforms.get(e)) {
                let pos_delta = target_transform.pos - transform.pos;
                let distance = pos_delta.length();
                if distance > follow.follow_distance {
                    target.target = None;
                    movement.target_acceleration_normal = Vec2f::zero();
                    debug!("{:?} lost its target: target too far.", e);
                } else {
                    let safe_delta = pos_delta.try_normalize().unwrap_or_default()
                        * (distance - follow.keep_distance);
                    let brake_factor = if distance >= follow.keep_distance {
                        // slowly approach
                        if (pos_delta - movement.velocity * 0.33).length() < follow.keep_distance {
                            0.0
                        } else {
                            1.0
                        }
                    } else {
                        // get out as fast as possible
                        1.0
                    };
                    let normalized = safe_delta.try_normalize().unwrap_or_default();
                    movement.target_acceleration_normal = normalized * brake_factor;
                }
            }
        }
    }
}
