use super::super::{component::*, resource::*};
use crate::{attack::AttackPatternData, math::*};
use nphysics2d::{
    algebra::ForceType,
    math::{Force, Isometry},
    object::{Body, RigidBody},
};
use specs::{
    storage::ComponentEvent, BitSet, Entities, Join, Read, ReadExpect, ReadStorage, ReaderId, System, SystemData,
    World, WorldExt, Write, WriteExpect, WriteStorage, Entity
};
use std::ops::DerefMut;

pub struct WeaponrySystem;
impl<'a> System<'a> for WeaponrySystem {
    type SystemData = (
        Read<'a, DeltaTime>,
        Write<'a, SpawnQueue>,
        WriteExpect<'a, PhysicWorld>,
        ReadStorage<'a, Physic>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Weaponry>,
        WriteStorage<'a, WeaponProperties>,
        ReadStorage<'a, WeaponAttack>,
    );

    fn run(
        &mut self,
        (dt, mut spawn_queue, mut pworld, physics, factions, transforms, mut weaponries, mut props, attacks): Self::SystemData,
    ) {
        for (transform, weaponry, faction_opt, physics_opt) in
            (&transforms, &mut weaponries, (&factions).maybe(), (&physics).maybe()).join()
        {
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
                        let mut data = AttackPatternData {
                            shooter_faction: faction_opt.map(|f| &f.id),
                            shooter_body: physics_opt
                                .and_then(|p| pworld.bodies.get_mut(p.body))
                                .and_then(|b| b.downcast_mut::<RigidBody<f32>>()),
                            shooting_at: transform.pos,
                            prop: prop,
                            projectiles: spawn_queue.deref_mut(),
                        };
                        attack.pattern.attack(&mut data);
                        prop.cooldown = prop.cooldown_time;
                        prop.clip -= 1;
                    } else {
                        prop.cooldown = (prop.cooldown - dt.0.as_secs_f32()).max(0.0);
                    }
                }
            }
        }
    }
}
pub struct ProjectileSystem;
impl<'a> System<'a> for ProjectileSystem {
    type SystemData =
        (Entities<'a>, ReadExpect<'a, PhysicWorld>, WriteStorage<'a, HealthPool>, ReadStorage<'a, DamageDealer>);

    fn run(&mut self, (entities, physic_world, mut hpools, ddealers): Self::SystemData) {
        use nphysics2d::ncollide2d::query::Proximity;
        for proximity in physic_world.geometry_world.proximity_events() {
            if proximity.new_status == Proximity::Intersecting {
                let (entity1, entity2) = (
                    physic_world.entity_for_collider(&proximity.collider1).unwrap(),
                    physic_world.entity_for_collider(&proximity.collider2).unwrap(),
                );

                let (mut hpool, ddealer, dealer_e) =
                    if let (Some(hpool), Some(ddealer)) = (hpools.get_mut(*entity1), ddealers.get(*entity2)) {
                        (hpool, ddealer, entity2)
                    } else if let (Some(hpool), Some(ddealer)) = (hpools.get_mut(*entity2), ddealers.get(*entity1)) {
                        (hpool, ddealer, entity1)
                    } else {
                        continue;
                    };

                hpool.hp = hpool.hp.saturating_sub(ddealer.damage);
                entities.delete(*dealer_e).unwrap();
            }
        }
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

pub struct DistanceCounterSystem;
impl<'a> System<'a> for DistanceCounterSystem {
    type SystemData = (WriteStorage<'a, DistanceCounter>, ReadStorage<'a, Transform>);

    fn run(&mut self, (mut counters, transforms): Self::SystemData) {
        for (counter, transform) in (&mut counters, &transforms).join() {
            if let Some(last) = counter.last_pos {
                let d = (transform.pos - last).length();
                counter.distance += d;
            }
            counter.last_pos.replace(transform.pos);
        }
    }
}

pub struct DistanceLimitingSystem;
impl<'a> System<'a> for DistanceLimitingSystem {
    type SystemData = (Entities<'a>, ReadStorage<'a, DistanceCounter>, ReadStorage<'a, DistanceLimited>);

    fn run(&mut self, (entities, counters, limits): Self::SystemData) {
        for (e, counter, limit) in (&entities, &counters, &limits).join() {
            if counter.distance >= limit.limit {
                entities.delete(e).unwrap();
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

#[derive(Default)]
pub struct SpriteDamageBlinkSystem {
    reader_id: Option<ReaderId<ComponentEvent>>,
    modified: BitSet,
}
impl<'a> System<'a> for SpriteDamageBlinkSystem {
    type SystemData = (Entities<'a>, WriteStorage<'a, SpriteBlink>, ReadStorage<'a, HealthPool>);

    fn run(&mut self, (entities, mut blinks, hpools): Self::SystemData) {
        let mut remove_queue: Vec<Entity> = vec![];
        for (e, blink) in (&entities, &mut blinks).join() {
            if blink.frames_left == 0 {
                remove_queue.push(e);
            } else {
                blink.frames_left -= 1;
            }
        }
        for e in remove_queue {
            blinks.remove(e);
        }

        self.modified.clear();
        for event in hpools.channel().read(self.reader_id.as_mut().unwrap()) {
            match event {
                ComponentEvent::Modified(id) => {
                    self.modified.add(*id);
                },
                _ => (),
            };
        }

        for (e, _) in (&entities, &self.modified).join() {
            blinks.insert(e, SpriteBlink { frames_left: 4 }).unwrap();
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.reader_id = Some(world.write_storage::<HealthPool>().register_reader());
    }
}
