use super::super::{component::*, resource::*, tag};
use crate::{
    assets::AssetManager,
    attack::{AttackPatternData, ProjectileData},
    entity, item,
    math::*,
    particle, read_event,
};
use itertools::Itertools;
use nphysics2d::{
    algebra::ForceType,
    math::{Force, Isometry},
    object::{Body, RigidBody},
};
use rand::{distributions::weighted::alias_method::WeightedIndex, prelude::*};
use specs::{
    storage::ComponentEvent, BitSet, Entities, Entity, Join, LazyUpdate, Read, ReadExpect, ReadStorage, ReaderId,
    System, SystemData, World, WorldExt, Write, WriteExpect, WriteStorage,
};
use std::{
    collections::BTreeMap as Map,
    ops::{Deref, DerefMut},
};

pub struct ShotsDodgerSystem;
impl<'a> System<'a> for ShotsDodgerSystem {
    type SystemData =
        (Entities<'a>, WriteStorage<'a, DamageReciever>, WriteStorage<'a, AvoidShots>, WriteStorage<'a, Transform>);

    fn run(&mut self, (entities, mut dmg_recs, mut dodgers, mut transforms): Self::SystemData) {
        let mut to_remove: Vec<_> = vec![];
        for (e, dmg_rec, avoid, transform) in (&entities, &mut dmg_recs, &mut dodgers, &mut transforms).join() {
            if dmg_rec.damage_queue.iter().any(|(_, dmg_type)| dmg_rec.damage_immunity[*dmg_type].is_none()) {
                if avoid.count > 0 {
                    let mut rng = thread_rng();
                    avoid.count -= 1;
                    transform.pos += Vec2f::new(rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0)).normalize() * 50.0;
                    for damage_type in &DAMAGE_TYPES {
                        dmg_rec.update_immunity(*damage_type, 0.5);
                    }
                    dmg_rec.damage_queue.clear();
                } else {
                    to_remove.push(e);
                }
            }
        }
        for e in to_remove {
            dodgers.remove(e);
        }
    }
}

pub struct ConsumablesSystem;
impl<'a> System<'a> for ConsumablesSystem {
    type SystemData = (Entities<'a>, Read<'a, DeltaTime>, Read<'a, LazyUpdate>, WriteStorage<'a, Consumer>);

    fn run(&mut self, (entities, dt, update, mut consumers): Self::SystemData) {
        let dt = dt.0.as_secs_f32();
        for (e, consumer) in (&entities, &mut consumers).join() {
            for handle in &mut consumer.handles {
                handle.time += dt;
            }
            consumer.handles.retain(|handle| !handle.behaviour.update(dt, handle.time, e, &update));
        }
    }
}

pub struct ShapeshifterSystem<'a>(pub &'a mut ggez::Context);
impl<'a> System<'a> for ShapeshifterSystem<'a> {
    type SystemData = (
        Entities<'a>,
        Read<'a, DeltaTime>,
        Read<'a, LazyUpdate>,
        WriteExpect<'a, AssetManager>,
        WriteStorage<'a, Shapeshifter>,
    );

    fn run(&mut self, (entities, dt, update, mut assets, mut shapeshifters): Self::SystemData) {
        let dt = dt.0.as_secs_f32();
        for (e, shapeshifter) in (&entities, &mut shapeshifters).join() {
            let form_time = shapeshifter.forms[shapeshifter.current].time();
            if shapeshifter.time > form_time {
                let next = (shapeshifter.current + 1) % shapeshifter.forms.len();
                shapeshifter.forms[shapeshifter.current].on_end(e, update.deref(), (&mut self.0, &mut assets));
                shapeshifter.forms[next].on_begin(e, update.deref(), (&mut self.0, &mut assets));
                shapeshifter.current = next;
                shapeshifter.time = 0.0;
            }
            update.exec(move |world| {
                let mut shapeshifters = world.write_storage::<Shapeshifter>();
                if let Some(mut shapeshifter) = shapeshifters.get_mut(e) {
                    if shapeshifter.forms[shapeshifter.current].can_update(e, &world) {
                        shapeshifter.time += dt;
                    }
                }
            });
        }
    }
}

pub struct ParticlesSystem;
impl<'a> System<'a> for ParticlesSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, DeltaTime>,
        WriteStorage<'a, ParticleProperties>,
        ReadStorage<'a, SharedParticleDef>,
        WriteStorage<'a, tag::PendingDestruction>,
    );

    fn run(&mut self, (entities, dt, mut properties, defs, mut to_destruct): Self::SystemData) {
        for (e, prop, def) in (&entities, &mut properties, &defs).join() {
            prop.frame_time += dt.0.as_secs_f32();
            if prop.frame_time >= def.time_per_frame {
                prop.current_frame += 1;
            }

            if prop.current_frame >= def.frames {
                to_destruct.insert(e, tag::PendingDestruction).unwrap();
            }
        }
    }
}

pub struct WeaponrySystem;
impl<'a> System<'a> for WeaponrySystem {
    type SystemData = (
        Read<'a, DeltaTime>,
        Write<'a, SpawnQueue>,
        WriteExpect<'a, PhysicWorld>,
        ReadStorage<'a, Physic>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, DamageReciever>,
        WriteStorage<'a, Weaponry>,
        WriteStorage<'a, WeaponProperties>,
        ReadStorage<'a, WeaponAttack>,
        ReadStorage<'a, tag::LastShot>,
        ReadStorage<'a, tag::PendingDestruction>,
    );

    fn run(
        &mut self,
        (
            dt,
            mut spawn_queue,
            mut pworld,
            physics,
            factions,
            transforms,
            mut dmg_recievers,
            mut weaponries,
            mut props,
            attacks,
            last_shots,
            to_destruct,
        ): Self::SystemData,
    ) {
        fn reload(prop: &mut WeaponProperties, dt: f32) {
            if prop.clip == 0 {
                prop.reloading += dt;
                if prop.reloading >= prop.reloading_time {
                    prop.reloading = 0.0;
                    prop.cooldown = 0.0;
                    prop.clip = prop.clip_size;
                }
            }
        }

        for (transform, weaponry, faction_opt, physics_opt, dmg_rec_opt, last_shot_opt, to_destruct_opt) in (
            &transforms,
            &mut weaponries,
            (&factions).maybe(),
            (&physics).maybe(),
            (&mut dmg_recievers).maybe(),
            (&last_shots).maybe(),
            (&to_destruct).maybe(),
        )
            .join()
        {
            if let Some(mut prop) = weaponry.secondary.and_then(|w| props.get_mut(w)) {
                if prop.passive_reloading {
                    reload(&mut prop, dt.0.as_secs_f32());
                }
            }
            if let Some((Some(mut prop), Some(attack))) = weaponry.primary.map(|w| (props.get_mut(w), attacks.get(w))) {
                // handle reloading
                reload(&mut prop, dt.0.as_secs_f32());

                // shot if cooled
                if prop.cooldown == 0.0 {
                    if (prop.is_shooting || (last_shot_opt.is_some() && to_destruct_opt.is_some())) && prop.clip > 0 {
                        let mut data = AttackPatternData {
                            shooter_faction: faction_opt.map(|f| &f.id),
                            shooter_body: physics_opt
                                .and_then(|p| pworld.bodies.get_mut(p.body))
                                .and_then(|b| b.downcast_mut::<RigidBody<f32>>()),
                            shooter_damage_reciever: dmg_rec_opt,
                            shooting_at: transform.pos.to_point(),
                            damage_multiplier: weaponry.damage_multiplier,
                            prop: prop,
                            projectiles: spawn_queue.deref_mut(),
                        };
                        attack.pattern.attack(&mut data);
                        prop.cooldown = prop.cooldown_time;
                        prop.clip -= 1;
                    }
                } else {
                    prop.cooldown = (prop.cooldown - dt.0.as_secs_f32()).max(0.0);
                }
            }
        }
    }
}
pub struct ProjectileSystem;
impl ProjectileSystem {
    fn comps_to_data<'a>(
        projectile: &'a Projectile,
        distance: &'a DistanceCounter,
        transform: &'a Transform,
        spawn_queue: &'a mut SpawnQueue,
    ) -> ProjectileData<'a> {
        ProjectileData {
            asset: projectile.def.asset.as_ref(),
            damage: projectile.def.damage,
            velocity: projectile.def.velocity,
            distance_traveled: distance.distance,
            pos: transform.pos.to_point(),
            size: projectile.def.size,
            ignore_groups: &projectile.def.ignore_groups,
            projectiles: spawn_queue,
        }
    }
}
impl<'a> System<'a> for ProjectileSystem {
    type SystemData = (
        WriteExpect<'a, SpawnQueue>,
        ReadExpect<'a, PhysicWorld>,
        ReadStorage<'a, DistanceCounter>,
        ReadStorage<'a, Projectile>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, DamageReciever>,
        ReadStorage<'a, DamageDealer>,
        WriteStorage<'a, tag::PendingDestruction>,
    );

    fn run(
        &mut self,
        (
            mut spawn_queue,
            physic_world,
            distances,
            projectiles,
            transforms,
            mut dmg_recievers,
            dmg_dealers,
            mut to_destruct,
        ): Self::SystemData,
    ) {
        let per_entity = physic_world
            .geometry_world
            .proximity_pairs(&physic_world.colliders, true)
            .filter_map(|(_, collider1, _, collider2, _, _)| {
                collider1
                    .user_data()
                    .and_then(|d| d.downcast_ref::<Entity>())
                    .into_iter()
                    .chain(collider2.user_data().and_then(|d| d.downcast_ref::<Entity>()))
                    .collect_tuple::<(&Entity, &Entity)>()
            })
            .dedup_by(|t1, t2| (t1.0 == t2.0 && t1.1 == t2.1) || (t1.1 == t2.0 && t1.0 == t2.1));

        for (entity1, entity2) in per_entity {
            let (dmg_rec, dmg_deal, projectile, deal_e) = if let (Some(dmg_rec), Some(dmg_deal), Some(projectile)) =
                (dmg_recievers.get_mut(*entity1), dmg_dealers.get(*entity2), projectiles.get(*entity2))
            {
                (dmg_rec, dmg_deal, projectile, entity2)
            } else if let (Some(dmg_rec), Some(dmg_deal), Some(projectile)) =
                (dmg_recievers.get_mut(*entity2), dmg_dealers.get(*entity1), projectiles.get(*entity1))
            {
                (dmg_rec, dmg_deal, projectile, entity1)
            } else {
                continue;
            };

            dmg_rec.damage_queue.push((dmg_deal.damage, dmg_deal.damage_type));
            let consumed = if let (Some(behaviour), Some(distance), Some(transform)) =
                (&projectile.def.behaviour, distances.get(*deal_e), transforms.get(*deal_e))
            {
                let mut data = Self::comps_to_data(&projectile, &distance, &transform, &mut spawn_queue);
                behaviour.on_hit(&mut data)
            } else {
                true
            };
            if consumed {
                to_destruct.insert(*deal_e, tag::PendingDestruction).unwrap();
            }
        }

        for (distance, transform, projectile) in (&distances, &transforms, &projectiles).join() {
            if distance.distance >= projectile.def.distance {
                if let Some(behaviour) = &projectile.def.behaviour {
                    let mut data = Self::comps_to_data(&projectile, &distance, &transform, &mut spawn_queue);
                    behaviour.on_end(&mut data);
                } else {
                    spawn_queue.0.push_back(SpawnItem::Particle(particle::ID::Splash, transform.pos.to_point()));
                }
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
                body.set_linear_damping((velocity_len / movement.max_velocity).max(1.0));

                // velocity soft-cap
                if velocity_len < movement.max_velocity {
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
        read_event!(transforms, self.reader_id.as_mut().unwrap(); Modified => self.modified);

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

#[derive(Default)]
pub struct ShootTargetSystem {
    reader_id: Option<ReaderId<ComponentEvent>>,
    removed: BitSet,
}
impl<'a> System<'a> for ShootTargetSystem {
    type SystemData = (
        WriteStorage<'a, WeaponProperties>,
        ReadStorage<'a, Weaponry>,
        ReadStorage<'a, Target>,
        ReadStorage<'a, ShootTarget>,
        ReadStorage<'a, Transform>,
    );

    fn run(&mut self, (mut wpn_props, weaponries, targets, shoot_targets, transforms): Self::SystemData) {
        for (transform, weaponry, target, shoot_target) in (&transforms, &weaponries, &targets, &shoot_targets).join() {
            if let (Some(target), Some(wpn_prop)) = (target.target, weaponry.primary.and_then(|w| wpn_props.get_mut(w)))
            {
                let target_transform = transforms.get(target).unwrap();
                let area = Circle2f::new(transform.pos.to_point(), shoot_target.radius);
                if area.contains(target_transform.pos.to_point()) {
                    wpn_prop.is_shooting = true;
                    wpn_prop.target_pos = target_transform.pos.to_point();
                } else {
                    wpn_prop.is_shooting = false;
                }
            }
        }

        read_event!(shoot_targets, self.reader_id.as_mut().unwrap(); Removed => self.removed);
        for (weaponry, _) in (&weaponries, &self.removed).join() {
            if let Some(wpn_prop) = weaponry.primary.and_then(|w| wpn_props.get_mut(w)) {
                wpn_prop.is_shooting = false;
            }
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.reader_id = Some(world.write_storage::<ShootTarget>().register_reader());
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
            for (e, mut target, search, search_transform) in
                (&entities, &mut targets.restrict_mut(), &searches, &transforms).join()
            {
                if target.get_unchecked().target.is_some() {
                    continue;
                }

                let area = Circle2f::new(search_transform.pos.to_point(), search.radius);
                if search.from_factions.contains(&faction.id) && area.contains(transform.pos.to_point()) {
                    log::debug!("{:?} found new target {:?}", e, e_target);
                    target.get_mut_unchecked().target = Some(e_target)
                }
            }
        }
    }
}

#[derive(Default)]
pub struct FollowTargetSystem {
    reader_id: Option<ReaderId<ComponentEvent>>,
    removed: BitSet,
}
impl<'a> System<'a> for FollowTargetSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, FollowTarget>,
        WriteStorage<'a, Target>,
        WriteStorage<'a, Movement>,
        ReadStorage<'a, Transform>,
    );

    fn run(&mut self, (entities, follows, mut targets, mut movements, transforms): Self::SystemData) {
        for (e, follow, mut target, movement, transform) in
            (&entities, &follows, &mut targets.restrict_mut(), &mut movements, &transforms).join()
        {
            if let Some(target_transform) = target.get_unchecked().target.and_then(|e| transforms.get(e)) {
                let pos_delta = target_transform.pos - transform.pos;
                let distance = pos_delta.length();
                if distance > follow.follow_distance {
                    target.get_mut_unchecked().target = None;
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

        read_event!(follows, self.reader_id.as_mut().unwrap(); Removed => self.removed);
        for (movement, _) in (&mut movements, &self.removed).join() {
            movement.target_acceleration_normal.x = 0.0;
            movement.target_acceleration_normal.y = 0.0;
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.reader_id = Some(world.write_storage::<FollowTarget>().register_reader());
    }
}

pub struct DistanceCounterSystem;
impl<'a> System<'a> for DistanceCounterSystem {
    type SystemData = (WriteStorage<'a, DistanceCounter>, ReadStorage<'a, Transform>);

    fn run(&mut self, (mut counters, transforms): Self::SystemData) {
        for (counter, transform) in (&mut counters, &transforms).join() {
            let d = (transform.pos - counter.last_pos.unwrap_or(transform.pos)).length();
            counter.distance += d;
            counter.last_pos.replace(transform.pos);
        }
    }
}

pub struct DistanceLimitingSystem;
impl<'a> System<'a> for DistanceLimitingSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, DistanceCounter>,
        ReadStorage<'a, DistanceLimited>,
        WriteStorage<'a, tag::PendingDestruction>,
    );

    fn run(&mut self, (entities, counters, limits, mut to_destruct): Self::SystemData) {
        for (e, counter, limit) in (&entities, &counters, &limits).join() {
            if counter.distance >= limit.limit {
                to_destruct.insert(e, tag::PendingDestruction).unwrap();
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

pub struct ContainerSinkSystem;
impl<'a> System<'a> for ContainerSinkSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, SpawnQueue>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Inventory>,
        ReadStorage<'a, tag::Container>,
        WriteStorage<'a, tag::PendingDestruction>,
    );

    fn run(
        &mut self,
        (entities, mut spawn_queue, transforms, inventories, containers, mut to_destruct): Self::SystemData,
    ) {
        for (e, transform, inventory, _) in (&entities, &transforms, &inventories, &containers).join() {
            if !inventory.content.have_some() {
                to_destruct.insert(e, tag::PendingDestruction).unwrap();
                spawn_queue.0.push_back(SpawnItem::Particle(particle::ID::MediumSplash, transform.pos.to_point()));
            }
        }
    }
}

pub struct ExplodeOnDeathSystem;
impl<'a> System<'a> for ExplodeOnDeathSystem {
    type SystemData = (
        WriteExpect<'a, SpawnQueue>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, tag::PendingDestruction>,
    );

    fn run(&mut self, (mut spawn_queue, faction, transform, to_destruct): Self::SystemData) {
        for (faction, transform, _) in (&faction, &transform, &to_destruct).join() {
            match faction.id {
                FactionId::Crabs | FactionId::Mythical => {
                    spawn_queue.0.push_back(SpawnItem::Particle(particle::ID::MediumSplash, transform.pos.to_point()));
                },
                FactionId::Pirates | FactionId::Good => {
                    spawn_queue.0.push_back(SpawnItem::Particle(particle::ID::Explosion, transform.pos.to_point()));
                },
            }
        }
    }
}

macro_rules! add_drops_from_group {
    ($weight:expr; $group:expr => $map:expr) => {
        if $weight > 0 {
            for item in &$group {
                *$map.entry(*item).or_insert(0) += $weight;
            }
        }
    };
}
pub struct LootGenerateSystem;
impl<'a> System<'a> for LootGenerateSystem {
    type SystemData = (
        WriteExpect<'a, SpawnQueue>,
        ReadStorage<'a, SharedDropTable>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, tag::PendingDestruction>,
    );

    fn run(&mut self, (mut spawn_queue, drops, transform, to_destruct): Self::SystemData) {
        for (drop, transform, _) in (&drops, &transform, &to_destruct).join() {
            let mut rng = thread_rng();
            if rng.gen::<f32>() <= drop.drop_chance {
                if rng.gen_range(0, 4) == 0 {
                    spawn_queue.0.push_back(SpawnItem::Entity(entity::ID::Mimic, transform.pos.to_point(), vec![]));
                } else {
                    let mut drop_map: Map<item::ID, u16> = Map::default();
                    add_drops_from_group!(drop.any_common; item::ANY_COMMON => drop_map);
                    add_drops_from_group!(drop.any_rare; item::ANY_RARE => drop_map);
                    add_drops_from_group!(drop.any_legendary; item::ANY_LEGENDARY => drop_map);
                    for (item, weight) in &drop.assigned_drops {
                        *drop_map.entry(*item).or_insert(0) += weight;
                    }
                    let drop_arr = drop_map.into_iter().collect_vec();
                    let dist = WeightedIndex::new(drop_arr.iter().map(|item| item.1).collect()).unwrap();
                    let new_drop = drop_arr[dist.sample(&mut rng)].0;
                    log::debug!("Spawning new lootbox with {:?}", new_drop);
                    spawn_queue
                        .0
                        .push_back(SpawnItem::Entity(entity::ID::Lootbox, transform.pos.to_point(), vec![new_drop]));
                }
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
        read_event!(directionals, self.reader_id.as_mut().unwrap(); Modified => self.modified);

        for (direction, physic, _) in (&directionals, &mut physics, &self.modified).join() {
            if let CollideShapeHandle::Directional { north, east, south, west } = &physic.colliders.real.1 {
                let collider = world.colliders.get_mut(physic.colliders.real.0).unwrap();
                collider.set_shape(directional!(direction.direction => north, east, south, west).clone());
            }
            if let Some(hitbox) = &physic.colliders.hitbox {
                if let CollideShapeHandle::Directional { north, east, south, west } = &hitbox.1 {
                    let collider = world.colliders.get_mut(hitbox.0).unwrap();
                    collider.set_shape(directional!(direction.direction => north, east, south, west).clone());
                }
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

        read_event!(hpools, self.reader_id.as_mut().unwrap(); Modified => self.modified);

        for (e, _) in (&entities, &self.modified).join() {
            blinks.insert(e, SpriteBlink { frames_left: 4 }).unwrap();
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.reader_id = Some(world.write_storage::<HealthPool>().register_reader());
    }
}
