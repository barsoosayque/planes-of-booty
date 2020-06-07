use crate::{
    ecs::component::{CollisionGroup, DamageReciever, DamageType, FactionId, WeaponProperties},
    math::*,
    particle,
};
use nphysics2d::{
    algebra::ForceType,
    math::Force,
    object::{Body, RigidBody},
};
use rand::distributions::{uniform::Uniform, Distribution};

pub trait ProjectileBuilder {
    fn projectile(&mut self, def: ProjectileDef);
    fn particle(&mut self, particle: particle::ID, pos: Point2f);
}

pub struct AttackPatternData<'a> {
    pub shooting_at: Point2f,
    pub shooter_faction: Option<&'a FactionId>,
    pub shooter_body: Option<&'a mut RigidBody<f32>>,
    pub shooter_damage_reciever: Option<&'a mut DamageReciever>,
    pub damage_multiplier: f32,
    pub prop: &'a mut WeaponProperties,
    pub projectiles: &'a mut dyn ProjectileBuilder,
}

pub struct ProjectileData<'a> {
    pub asset: Option<&'a String>,
    pub damage: (u32, DamageType),
    pub velocity: Vec2f,
    pub distance_traveled: f32,
    pub pos: Point2f,
    pub size: Size2f,
    pub ignore_groups: &'a Vec<CollisionGroup>,
    pub projectiles: &'a mut dyn ProjectileBuilder,
}

#[derive(Default)]
pub struct ProjectileDef {
    pub asset: Option<String>,
    pub damage: (u32, DamageType),
    pub rotate_projectile: bool,
    pub velocity: Vec2f,
    pub distance: f32,
    pub pos: Point2f,
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
    if let Some(id) = shooter {
        match id {
            &FactionId::Good => vec![CollisionGroup::Players],
            &FactionId::Pirates | &FactionId::Crabs | &FactionId::Mythical => vec![CollisionGroup::Enemies],
        }
    } else {
        vec![]
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

pub struct Ram {
    pub power: f32,
}
impl AttackPattern for Ram {
    fn description(&self) -> &str { "Throw yourself into the battle with full impact damage negotiation." }

    fn attack(&self, data: &mut AttackPatternData) {
        let shooting_normal = (data.prop.target_pos - data.shooting_at).normalize();
        if let Some(body) = &mut data.shooter_body {
            let throw = shooting_normal * self.power;
            body.apply_force(0, &Force::linear([throw.x, throw.y].into()), ForceType::VelocityChange, true);
        }
        if let Some(dmg_rec) = &mut data.shooter_damage_reciever {
            dmg_rec.update_immunity(DamageType::Impact, 0.8);
        }
    }
}

pub struct Lightning;
impl AttackPattern for Lightning {
    fn description(&self) -> &str { "Area of effect attack with lightning damage type." }

    fn attack(&self, data: &mut AttackPatternData) {
        let def = ProjectileDef {
            damage: ((data.prop.damage as f32 * data.damage_multiplier) as u32, DamageType::Lightning),
            pos: data.prop.target_pos,
            size: Size2f::new(150.0, 150.0),
            ignore_groups: exclude_shooter(data.shooter_faction),
            behaviour: Some(Box::new(Self)),
            ..ProjectileDef::default()
        };
        data.projectiles.projectile(def);

        for i in 0..12 {
            let (sin, cos) = Angle2f::radians(360.0 * i as f32).sin_cos();
            data.projectiles.particle(
                particle::ID::Electro,
                Point2f::new(data.prop.target_pos.x + sin * 75.0, data.prop.target_pos.y + cos * 75.0),
            );
        }
    }
}
impl ProjectileBehaviour for Lightning {
    fn on_hit<'a>(&self, _: &mut ProjectileData<'a>) -> bool { false }
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
        let shooting_normal = (data.prop.target_pos - data.shooting_at).normalize();
        let def = ProjectileDef {
            asset: Some("/sprites/projectile/simple.png".to_owned()),
            damage: ((data.prop.damage as f32 * data.damage_multiplier) as u32, DamageType::Physical),
            velocity: with_accuracy(shooting_normal, data.prop.accuracy) * Self::PROJECTILE_VELOCITY_FLAT,
            distance: Self::DISTANCE,
            pos: data.shooting_at,
            size: Size2f::new(10.0, 10.0),
            ignore_groups: exclude_shooter(data.shooter_faction),
            ..ProjectileDef::default()
        };
        data.projectiles.projectile(def);
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
        let shooting_normal = (data.prop.target_pos - data.shooting_at).normalize();
        let def = ProjectileDef {
            asset: Some("/sprites/projectile/dark.png".to_owned()),
            damage: ((data.prop.damage as f32 * data.damage_multiplier) as u32, DamageType::Physical),
            velocity: with_accuracy(shooting_normal, data.prop.accuracy) * Self::PROJECTILE_VELOCITY_FLAT,
            distance: Self::DISTANCE,
            pos: data.shooting_at,
            size: Size2f::new(8.0, 8.0),
            ignore_groups: exclude_shooter(data.shooter_faction),
            behaviour: Some(Box::new(Self)),
            ..ProjectileDef::default()
        };
        data.projectiles.projectile(def);
    }
}
impl ProjectileBehaviour for Railgun {
    fn on_hit<'a>(&self, _: &mut ProjectileData<'a>) -> bool { false }
}

pub struct Crossbow;
impl Crossbow {
    const DISTANCE: f32 = 350.0;
    const PROJECTILE_VELOCITY_FLAT: f32 = 400.0;
}
impl AttackPattern for Crossbow {
    fn description(&self) -> &str { "Allow for faster shooting with accuracity decrease." }

    fn attack(&self, data: &mut AttackPatternData) {
        let shooting_normal = (data.prop.target_pos - data.shooting_at).normalize();
        let def = ProjectileDef {
            asset: Some("/sprites/projectile/bolt.png".to_owned()),
            damage: ((data.prop.damage as f32 * data.damage_multiplier) as u32, DamageType::Physical),
            velocity: with_accuracy(shooting_normal, data.prop.accuracy) * Self::PROJECTILE_VELOCITY_FLAT,
            distance: Self::DISTANCE,
            pos: data.shooting_at,
            size: Size2f::new(15.0, 7.0),
            ignore_groups: exclude_shooter(data.shooter_faction),
            rotate_projectile: true,
            ..ProjectileDef::default()
        };
        data.projectiles.projectile(def);
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
        let shooting_normal = (data.prop.target_pos - data.shooting_at).normalize();
        if let Some(body) = &mut data.shooter_body {
            let recoil = shooting_normal * -Self::RECOIL;
            body.apply_force(0, &Force::linear([recoil.x, recoil.y].into()), ForceType::VelocityChange, true);
        }
        let def = ProjectileDef {
            asset: Some("/sprites/projectile/dark.png".to_owned()),
            damage: ((data.prop.damage as f32 * data.damage_multiplier) as u32, DamageType::Physical),
            velocity: with_accuracy(shooting_normal, data.prop.accuracy) * Self::PROJECTILE_VELOCITY_FLAT,
            distance: Self::DISTANCE,
            pos: data.shooting_at,
            size: Size2f::new(15.0, 15.0),
            ignore_groups: exclude_shooter(data.shooter_faction),
            ..ProjectileDef::default()
        };
        data.projectiles.projectile(def);
    }
}

pub struct Shotgun {
    pub projectile: &'static str,
    pub projectile_size: Size2f,
    pub rotate_projectile: bool,
    pub distance: f32,
    pub pellets: u8,
    pub recoil: f32,
}
impl Shotgun {
    const ANGLE_LEFT_RAD: f32 = -0.392687;
    const ANGLE_RIGHT_RAD: f32 = 0.392687;
    const PROJECTILE_VELOCITY_FLAT: f32 = 300.0;
}
impl AttackPattern for Shotgun {
    fn description(&self) -> &str { "Juicy multi-projectile shots." }

    fn attack(&self, data: &mut AttackPatternData) {
        let shooting_normal = (data.prop.target_pos - data.shooting_at).normalize();
        if let Some(body) = &mut data.shooter_body {
            let recoil = shooting_normal * -self.recoil;
            body.apply_force(0, &Force::linear([recoil.x, recoil.y].into()), ForceType::VelocityChange, true);
        }
        let (left, right) = (Angle2f::radians(Self::ANGLE_LEFT_RAD), Angle2f::radians(Self::ANGLE_RIGHT_RAD));
        let corrected = with_accuracy(shooting_normal, data.prop.accuracy);
        for i in 0..self.pellets {
            let angle_offset = left.lerp(right, i as f32 / (self.pellets as f32 - 1.0));
            let pellet_normal = with_angle_offset(corrected, angle_offset);
            let def = ProjectileDef {
                asset: Some(self.projectile.to_owned()),
                rotate_projectile: self.rotate_projectile,
                damage: ((data.prop.damage as f32 * data.damage_multiplier) as u32, DamageType::Physical),
                velocity: pellet_normal * Self::PROJECTILE_VELOCITY_FLAT,
                distance: self.distance,
                pos: data.shooting_at,
                size: self.projectile_size,
                ignore_groups: exclude_shooter(data.shooter_faction),
                ..ProjectileDef::default()
            };
            data.projectiles.projectile(def);
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
        let shooting_normal = (data.prop.target_pos - data.shooting_at).normalize();
        let def = ProjectileDef {
            asset: Some("/sprites/projectile/bullet.png".to_owned()),
            damage: ((data.prop.damage as f32 * data.damage_multiplier) as u32, DamageType::Physical),
            velocity: with_accuracy(shooting_normal, data.prop.accuracy) * Self::PROJECTILE_VELOCITY_FLAT,
            distance: Self::DISTANCE_FIRST,
            pos: data.shooting_at,
            size: Size2f::new(10.0, 8.0),
            ignore_groups: exclude_shooter(data.shooter_faction),
            behaviour: Some(Box::new(Self)),
            rotate_projectile: true,
            ..ProjectileDef::default()
        };
        data.projectiles.projectile(def);
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
                asset: Some(data.asset.unwrap().clone()),
                damage: data.damage,
                velocity: pellet_normal * Self::PROJECTILE_VELOCITY_FLAT,
                distance: Self::DISTANCE_SPLIT,
                pos: data.pos.clone(),
                size: data.size.clone(),
                ignore_groups: data.ignore_groups.clone(),
                rotate_projectile: true,
                ..ProjectileDef::default()
            };
            data.projectiles.projectile(def);
        }
    }
}
