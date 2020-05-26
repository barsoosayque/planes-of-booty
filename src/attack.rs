use crate::ecs::{component::*, resource::*};

pub struct AttackPatternData<'a> {
    pub prop: &'a mut WeaponProperties,
}

pub trait AttackPattern: Sync + Send {
    fn description(&self) -> &str;
    fn attack(&self, data: &mut AttackPatternData);
}

pub struct Slingshot;
impl AttackPattern for Slingshot {
    fn description(&self) -> &str {
        "Shoot with mediocre accuracy, mediocre damage and mediocre distance. What did you expect ?"
    }

    fn attack(&self, data: &mut AttackPatternData) {
        log::debug!("shoot shoot :)");
    }
}
