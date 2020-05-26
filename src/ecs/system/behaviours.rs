use super::super::{component::*, resource::*};
use crate::{math::*, attack::AttackPatternData};
use nphysics2d::{
    algebra::ForceType,
    math::{Force, Isometry},
    object::{Body, RigidBody},
};
use specs::{
    storage::ComponentEvent, BitSet, Entities, Join, Read, ReadStorage, ReaderId, System, SystemData, World, WorldExt,
    WriteExpect, WriteStorage,
};

pub struct WeaponrySystem;
impl<'a> System<'a> for WeaponrySystem {
    type SystemData = (Read<'a, DeltaTime>, WriteStorage<'a, Weaponry>, WriteStorage<'a, WeaponProperties>, ReadStorage<'a, WeaponAttack>);

    fn run(&mut self, (dt, mut weaponries, mut props, attacks): Self::SystemData) {
        for weaponry in (&mut weaponries).join() {
            if let Some((Some(mut prop), Some(attack))) = weaponry.primary.map(|w| (props.get_mut(w), attacks.get(w))) {
                // handle reloading
                if prop.clip == 0 {
                    prop.reloading += dt.0.as_secs_f32();
                    if prop.reloading >= prop.reloading_time {
                        prop.reloading = 0.0;
                        prop.clip = prop.clip_size;
                    }
                }

                // shot if cooled
                if prop.is_shooting && prop.clip > 0 {
                    if prop.cooldown == 0.0 {
                        let mut data = AttackPatternData { prop  };
                        attack.pattern.attack(&mut data);
                        prop.clip -= 1;
                    } else {
                        prop.cooldown = (prop.cooldown - dt.0.as_secs_f32()).max(0.0);
                    }
                }
            }
        }
    }
}

pub struct ProjectilesSystem;
impl<'a> System<'a> for ProjectilesSystem {
    type SystemData =
        (Entities<'a>, WriteStorage<'a, Weaponry>, WriteStorage<'a, WeaponProperties>, ReadStorage<'a, WeaponAttack>);

    fn run(&mut self, (mut _entities, mut _weaponries, mut _props, _attacks): Self::SystemData) {
        // TODO
    }
}

pub struct PhysicSystem;
impl<'a> System<'a> for PhysicSystem {
    type SystemData =
        (WriteStorage<'a, Transform>, WriteStorage<'a, Movement>, Read<'a, DeltaTime>, WriteExpect<'a, PhysicWorld>);

    fn run(&mut self, (mut transforms, mut movements, delta, mut world): Self::SystemData) {
        // set data before simulation
        for (e, body) in world.bodies_iter_mut() {
            if let Some(movement) = movements.get_mut(e) {
                let velocity_len = movement.velocity.length();
                // velocity soft-cap
                if velocity_len > movement.max_velocity {
                    body.set_linear_damping(0.9);
                } else {
                    body.set_linear_damping(0.98);

                    let acceleration = movement.target_acceleration_normal * movement.acceleration_flat;
                    let force = Force::linear([acceleration.x, acceleration.y].into());
                    body.apply_force(0, &force, ForceType::AccelerationChange, true);

                    // amount of target acceleration converted directly into
                    // raw velocity
                    // TODO: there should be a way to do this using physic engine itself
                    if movement.steering_difficulty < 1.0 {
                        let velocity_compensation = movement.target_acceleration_normal * velocity_len
                            - movement.velocity * movement.target_acceleration_normal.length();
                        let velocity = velocity_compensation * (1.0 - movement.steering_difficulty);
                        let force = Force::linear([velocity.x, velocity.y].into());
                        body.apply_force(0, &force, ForceType::VelocityChange, true);
                    }
                }
            }
        }

        // run simulation
        world.step(delta.0);

        // update components based on simulation results
        for (e, body) in world.bodies_iter() {
            transforms.set_event_emission(false);
            if let Some(transform) = transforms.get_mut(e) {
                let pos = body.position().translation.vector;
                transform.pos.x = pos[0];
                transform.pos.y = pos[1];
            }
            transforms.set_event_emission(true);

            if let Some(movement) = movements.get_mut(e) {
                let velocity = body.velocity().linear;
                movement.velocity.x = velocity[0];
                movement.velocity.y = velocity[1];
            }
        }
    }
}

#[derive(Default)]
pub struct PhysicTransformSyncSystem {
    reader_id: Option<ReaderId<ComponentEvent>>,
    modified: BitSet,
}
impl<'a> System<'a> for PhysicTransformSyncSystem {
    type SystemData = (ReadStorage<'a, Transform>, ReadStorage<'a, Physic>, WriteExpect<'a, PhysicWorld>);

    fn run(&mut self, (transforms, physics, mut world): Self::SystemData) {
        self.modified.clear();
        for event in transforms.channel().read(self.reader_id.as_mut().unwrap()) {
            match event {
                ComponentEvent::Modified(id) => {
                    self.modified.add(*id);
                },
                _ => (),
            };
        }

        for (transform, physic, _) in (&transforms, &physics, &self.modified).join() {
            log::debug!("Manually changing transform of body");
            let body = world.bodies.get_mut(physic.body).unwrap().downcast_mut::<RigidBody<f32>>().unwrap();
            body.set_position(Isometry::translation(transform.pos.x, transform.pos.y));
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.reader_id = Some(world.write_storage::<Transform>().register_reader());
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
            for (e, target, search, search_transform) in (&entities, &mut targets, &searches, &transforms).join() {
                if target.target.is_some() {
                    continue;
                }

                let area = Circle2f::new(search_transform.pos.to_point(), search.radius);
                if search.from_factions.contains(&faction.id) && area.contains(transform.pos.to_point()) {
                    log::debug!("{:?} found new target {:?}", e, e_target);
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

    fn run(&mut self, (entities, follows, mut targets, mut movements, transforms): Self::SystemData) {
        for (e, follow, target, movement, transform) in
            (&entities, &follows, &mut targets, &mut movements, &transforms).join()
        {
            if let Some(target_transform) = target.target.and_then(|e| transforms.get(e)) {
                let pos_delta = target_transform.pos - transform.pos;
                let distance = pos_delta.length();
                if distance > follow.follow_distance {
                    target.target = None;
                    movement.target_acceleration_normal = Vec2f::zero();
                    log::debug!("{:?} lost its target: target too far.", e);
                } else {
                    let safe_delta = pos_delta.try_normalize().unwrap_or_default() * (distance - follow.keep_distance);
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

pub struct DirectionalSystem;
impl<'a> System<'a> for DirectionalSystem {
    type SystemData = (WriteStorage<'a, Directional>, ReadStorage<'a, Movement>);

    fn run(&mut self, (mut directionals, movements): Self::SystemData) {
        for (mut directional, movement) in (&mut directionals.restrict_mut(), &movements).join() {
            let new_direction = Direction::from_vec2f(&movement.velocity);
            if new_direction != directional.get_unchecked().direction {
                directional.get_mut_unchecked().direction = new_direction;
            }
        }
    }
}

#[derive(Default)]
pub struct DirectionalCollidersSystem {
    reader_id: Option<ReaderId<ComponentEvent>>,
    modified: BitSet,
}
impl<'a> System<'a> for DirectionalCollidersSystem {
    type SystemData = (ReadStorage<'a, Directional>, WriteStorage<'a, Physic>, WriteExpect<'a, PhysicWorld>);

    fn run(&mut self, (directionals, mut physics, mut world): Self::SystemData) {
        self.modified.clear();
        for event in directionals.channel().read(self.reader_id.as_mut().unwrap()) {
            match event {
                ComponentEvent::Modified(id) => {
                    self.modified.add(*id);
                },
                _ => (),
            };
        }

        for (direction, physic, _) in (&directionals, &mut physics, &self.modified).join() {
            if let CollideShapeHandle::Directional { north, east, south, west } = &physic.collide.1 {
                let collider = world.colliders.get_mut(physic.collide.0).unwrap();
                collider.set_shape(directional!(direction.direction => north, east, south, west).clone());
            }
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.reader_id = Some(world.write_storage::<Directional>().register_reader());
    }
}
