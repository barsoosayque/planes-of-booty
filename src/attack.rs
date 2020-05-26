use crate::{
    ecs::component::{FactionId, CollisionGroup, WeaponProperties},
    math::*,
};
use rand::distributions::{uniform::Uniform, Distribution};

pub trait ProjectileBuilder {
    fn build(&mut self, def: ProjectileDef);
}

pub struct AttackPatternData<'a> {
    pub shooting_at: Vec2f,
    pub shooter_faction: Option<&'a FactionId>,
    pub prop: &'a mut WeaponProperties,
    pub projectiles: &'a mut dyn ProjectileBuilder,
}

#[derive(Debug, Clone)]
pub struct ProjectileDef {
    pub asset: String,
    pub damage: u32,
    pub velocity: Vec2f,
    pub distance: f32,
    pub pos: Vec2f,
    pub size: Size2f,
    pub ignore_groups: Vec<CollisionGroup>,
}

pub trait AttackPattern: Sync + Send {
    fn description(&self) -> &str;
    fn attack(&self, data: &mut AttackPatternData);
}

pub struct Slingshot;
impl Slingshot {
    const ACCURACITY: f32 = 0.9;
    const DAMAGE: u32 = 7;
    const DISTANCE: f32 = 400.0;
    const PROJECTILE_VELOCITY_FLAT: f32 = 250.0;
}
impl AttackPattern for Slingshot {
    fn description(&self) -> &str {
        "Shoot with mediocre accuracy, mediocre damage and mediocre distance. What did you expect ?"
    }

    fn attack(&self, data: &mut AttackPatternData) {
        let ignore_groups = match data.shooter_faction {
            Some(&FactionId::Good) => vec![CollisionGroup::Players],
            Some(&FactionId::Pirates) => vec![CollisionGroup::Enemies],
            _ => vec![]
        };

        let u = Uniform::new_inclusive(-1.0, 1.0);
        let mut rng = rand::thread_rng();
        let accuracy_lost = Vec2f::new(u.sample(&mut rng), u.sample(&mut rng));
        let def = ProjectileDef {
            asset: "/sprites/projectile/simple.png".to_owned(),
            damage: Self::DAMAGE,
            velocity: accuracy_lost.lerp(data.prop.shooting_normal, Self::ACCURACITY) * Self::PROJECTILE_VELOCITY_FLAT,
            distance: Self::DISTANCE,
            pos: data.shooting_at,
            size: Size2f::new(10.0, 10.0),
            ignore_groups,
        };
        data.projectiles.build(def);
    }
}
