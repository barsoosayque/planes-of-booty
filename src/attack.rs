use crate::{
    ecs::component::{CollisionGroup, FactionId, WeaponProperties},
    math::*,
};
use nphysics2d::{
    algebra::ForceType,
    math::Force,
    object::{Body, RigidBody},
};
use rand::distributions::{uniform::Uniform, Distribution};

pub trait ProjectileBuilder {
    fn build(&mut self, def: ProjectileDef);
}

pub struct AttackPatternData<'a> {
    pub shooting_at: Vec2f,
    pub shooter_faction: Option<&'a FactionId>,
    pub shooter_body: Option<&'a mut RigidBody<f32>>,
    pub prop: &'a mut WeaponProperties,
    pub projectiles: &'a mut dyn ProjectileBuilder,
}

pub struct ProjectileData<'a> {
    pub asset: &'a String,
    pub damage: u32,
    pub velocity: &'a Vec2f,
    pub distance_traveled: f32,
    pub pos: &'a Vec2f,
    pub size: &'a Size2f,
    pub ignore_groups: &'a Vec<CollisionGroup>,
    pub projectiles: &'a mut dyn ProjectileBuilder,
}

pub struct ProjectileDef {
    pub asset: String,
    pub damage: u32,
    pub velocity: Vec2f,
    pub distance: f32,
    pub pos: Vec2f,
    pub size: Size2f,
    pub ignore_groups: Vec<CollisionGroup>,
    pub behaviour: Option<Box<dyn ProjectileBehaviour>>,
}

pub trait AttackPattern: Sync + Send {
    fn description(&self) -> &str;
    fn attack(&self, data: &mut AttackPatternData);
}

pub trait ProjectileBehaviour: Sync + Send {
    fn on_end(&self, _data: &mut ProjectileData) {}
    fn on_hit(&self, _data: &mut ProjectileData) -> bool { true }
}

fn exclude_shooter(shooter: Option<&FactionId>) -> Vec<CollisionGroup> {
    match shooter {
        Some(&FactionId::Good) => vec![CollisionGroup::Players],
        Some(&FactionId::Pirates) => vec![CollisionGroup::Enemies],
        _ => vec![],
    }
}
fn with_accuracy(normal: Vec2f, accuracy: f32) -> Vec2f {
    let bound = 1.5707 * (1.0 - accuracy);
    let u = Uniform::new_inclusive(-bound, bound);
    let mut rng = rand::thread_rng();
    with_angle_offset(normal, Angle2f::radians(u.sample(&mut rng)))
}
fn with_angle_offset(normal: Vec2f, angle: Angle2f) -> Vec2f {
    let (s, c) = angle.sin_cos();
    Vec2f::new(normal.x * c - normal.y * s, normal.x * s + normal.y * c)
}

pub struct Slingshot;
impl Slingshot {
    const DISTANCE: f32 = 400.0;
    const PROJECTILE_VELOCITY_FLAT: f32 = 250.0;
}
impl AttackPattern for Slingshot {
    fn description(&self) -> &str {
        "Shoot with mediocre accuracy, mediocre damage and mediocre distance. What did you expect ?"
    }

    fn attack(&self, data: &mut AttackPatternData) {
        let def = ProjectileDef {
            asset: "/sprites/projectile/simple.png".to_owned(),
            damage: data.prop.damage,
            velocity: with_accuracy(data.prop.shooting_normal, data.prop.accuracy) * Self::PROJECTILE_VELOCITY_FLAT,
            distance: Self::DISTANCE,
            pos: data.shooting_at,
            size: Size2f::new(10.0, 10.0),
            ignore_groups: exclude_shooter(data.shooter_faction),
            behaviour: None,
        };
        data.projectiles.build(def);
    }
}

pub struct Railgun;
impl Railgun {
    const DISTANCE: f32 = 1000.0;
    const PROJECTILE_VELOCITY_FLAT: f32 = 1000.0;
}
impl AttackPattern for Railgun {
    fn description(&self) -> &str { "Penetration at extreme speed." }

    fn attack(&self, data: &mut AttackPatternData) {
        let def = ProjectileDef {
            asset: "/sprites/projectile/simple.png".to_owned(),
            damage: data.prop.damage,
            velocity: with_accuracy(data.prop.shooting_normal, data.prop.accuracy) * Self::PROJECTILE_VELOCITY_FLAT,
            distance: Self::DISTANCE,
            pos: data.shooting_at,
            size: Size2f::new(8.0, 8.0),
            ignore_groups: exclude_shooter(data.shooter_faction),
            behaviour: Some(Box::new(Self)),
        };
        data.projectiles.build(def);
    }
}
impl ProjectileBehaviour for Railgun {
    fn on_hit<'a>(&self, _: &mut ProjectileData<'a>) -> bool {
        false
    }
}

pub struct Crossbow;
impl Crossbow {
    const DISTANCE: f32 = 350.0;
    const PROJECTILE_VELOCITY_FLAT: f32 = 400.0;
}
impl AttackPattern for Crossbow {
    fn description(&self) -> &str { "Allow for faster shooting with accuracity decrease." }

    fn attack(&self, data: &mut AttackPatternData) {
        let def = ProjectileDef {
            asset: "/sprites/projectile/bolt.png".to_owned(),
            damage: data.prop.damage,
            velocity: with_accuracy(data.prop.shooting_normal, data.prop.accuracy) * Self::PROJECTILE_VELOCITY_FLAT,
            distance: Self::DISTANCE,
            pos: data.shooting_at,
            size: Size2f::new(15.0, 7.0),
            ignore_groups: exclude_shooter(data.shooter_faction),
            behaviour: None,
        };
        data.projectiles.build(def);
    }
}

pub struct Cannon;
impl Cannon {
    const DISTANCE: f32 = 200.0;
    const PROJECTILE_VELOCITY_FLAT: f32 = 200.0;
    const RECOIL: f32 = 400.0;
}
impl AttackPattern for Cannon {
    fn description(&self) -> &str { "Slow moving projectile which pushes you back at fire." }

    fn attack(&self, data: &mut AttackPatternData) {
        if let Some(body) = &mut data.shooter_body {
            let recoil = data.prop.shooting_normal * -Self::RECOIL;
            body.apply_force(0, &Force::linear([recoil.x, recoil.y].into()), ForceType::VelocityChange, true);
        }
        let def = ProjectileDef {
            asset: "/sprites/projectile/simple.png".to_owned(),
            damage: data.prop.damage,
            velocity: with_accuracy(data.prop.shooting_normal, data.prop.accuracy) * Self::PROJECTILE_VELOCITY_FLAT,
            distance: Self::DISTANCE,
            pos: data.shooting_at,
            size: Size2f::new(10.0, 10.0),
            ignore_groups: exclude_shooter(data.shooter_faction),
            behaviour: None,
        };
        data.projectiles.build(def);
    }
}

pub struct Shotgun {
    pub pellets: u8,
}
impl Shotgun {
    const ANGLE_LEFT_RAD: f32 = -0.392687;
    const ANGLE_RIGHT_RAD: f32 = 0.392687;
    const DISTANCE: f32 = 350.0;
    const PROJECTILE_VELOCITY_FLAT: f32 = 300.0;
}
impl AttackPattern for Shotgun {
    fn description(&self) -> &str { "Juicy multi-projectile shots." }

    fn attack(&self, data: &mut AttackPatternData) {
        let (left, right) = (Angle2f::radians(Self::ANGLE_LEFT_RAD), Angle2f::radians(Self::ANGLE_RIGHT_RAD));
        let corrected = with_accuracy(data.prop.shooting_normal, data.prop.accuracy);
        for i in 0..self.pellets {
            let angle_offset = left.lerp(right, i as f32 / (self.pellets as f32 - 1.0));
            let pellet_normal = with_angle_offset(corrected, angle_offset);
            let def = ProjectileDef {
                asset: "/sprites/projectile/simple.png".to_owned(),
                damage: data.prop.damage,
                velocity: pellet_normal * Self::PROJECTILE_VELOCITY_FLAT,
                distance: Self::DISTANCE,
                pos: data.shooting_at,
                size: Size2f::new(10.0, 10.0),
                ignore_groups: exclude_shooter(data.shooter_faction),
                behaviour: None,
            };
            data.projectiles.build(def);
        }
    }
}

pub struct Split;
impl Split {
    const ANGLE_LEFT_RAD: f32 = -0.196343;
    const ANGLE_RIGHT_RAD: f32 = 0.196343;
    const DISTANCE_FIRST: f32 = 200.0;
    const DISTANCE_SPLIT: f32 = 200.0;
    const PROJECTILE_VELOCITY_FLAT: f32 = 300.0;
}
impl AttackPattern for Split {
    fn description(&self) -> &str { "Shoots a projectile that splits mid-air." }

    fn attack(&self, data: &mut AttackPatternData) {
        let def = ProjectileDef {
            asset: "/sprites/projectile/simple.png".to_owned(),
            damage: data.prop.damage,
            velocity: with_accuracy(data.prop.shooting_normal, data.prop.accuracy) * Self::PROJECTILE_VELOCITY_FLAT,
            distance: Self::DISTANCE_FIRST,
            pos: data.shooting_at,
            size: Size2f::new(10.0, 10.0),
            ignore_groups: exclude_shooter(data.shooter_faction),
            behaviour: Some(Box::new(Self)),
        };
        data.projectiles.build(def);
    }
}
impl ProjectileBehaviour for Split {
    fn on_end<'a>(&self, data: &mut ProjectileData<'a>) {
        let (left, right) = (Angle2f::radians(Self::ANGLE_LEFT_RAD), Angle2f::radians(Self::ANGLE_RIGHT_RAD));
        let shooting_normal = data.velocity.normalize();
        for i in 0..4 {
            let angle_offset = left.lerp(right, i as f32 / 3.0);
            let pellet_normal = with_angle_offset(shooting_normal, angle_offset);
            let def = ProjectileDef {
                asset: data.asset.clone(),
                damage: data.damage,
                velocity: pellet_normal * Self::PROJECTILE_VELOCITY_FLAT,
                distance: Self::DISTANCE_SPLIT,
                pos: data.pos.clone(),
                size: data.size.clone(),
                ignore_groups: data.ignore_groups.clone(),
                behaviour: None,
            };
            data.projectiles.build(def);
        }
    }
}
