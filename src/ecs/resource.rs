use crate::{
    math::{Point2f, Vec2f},
    ui::*,
};
use ggez::input;
use nphysics2d::{
    force_generator::DefaultForceGeneratorSet,
    joint::DefaultJointConstraintSet,
    object::{Collider, DefaultBodyHandle, DefaultBodySet, DefaultColliderSet, RigidBody},
    world::{DefaultGeometricalWorld, DefaultMechanicalWorld},
};
use specs::Entity;
use std::collections::{HashSet, VecDeque as Queue};

#[derive(Default, Debug)]
pub struct DeltaTime(pub std::time::Duration);

pub struct PhysicWorld {
    pub mecha_world: DefaultMechanicalWorld<f32>,
    pub geometry_world: DefaultGeometricalWorld<f32>,
    pub bodies: DefaultBodySet<f32>,
    pub colliders: DefaultColliderSet<f32>,
    pub joint_constraints: DefaultJointConstraintSet<f32>,
    pub force_generators: DefaultForceGeneratorSet<f32>,
}
impl PhysicWorld {
    pub fn new(gravity: Vec2f) -> Self {
        use nphysics2d::nalgebra::Vector2;
        Self {
            mecha_world: DefaultMechanicalWorld::new(Vector2::new(gravity.x, gravity.y)),
            geometry_world: DefaultGeometricalWorld::new(),
            bodies: DefaultBodySet::new(),
            colliders: DefaultColliderSet::new(),
            joint_constraints: DefaultJointConstraintSet::new(),
            force_generators: DefaultForceGeneratorSet::new(),
        }
    }

    pub fn step(&mut self, delta: std::time::Duration) {
        self.mecha_world.set_timestep(delta.as_secs_f32());
        self.mecha_world.step(
            &mut self.geometry_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joint_constraints,
            &mut self.force_generators,
        );
    }

    pub fn bodies_iter(&self) -> impl Iterator<Item = (Entity, &RigidBody<f32>)> {
        self.bodies
            .iter()
            .filter_map(|b| b.1.downcast_ref::<RigidBody<f32>>())
            .map(|b| (*b.user_data().unwrap().downcast_ref::<Entity>().unwrap(), b))
    }

    pub fn bodies_iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut RigidBody<f32>)> {
        self.bodies
            .iter_mut()
            .filter_map(|b| b.1.downcast_mut::<RigidBody<f32>>())
            .map(|b| (*b.user_data().unwrap().downcast_ref::<Entity>().unwrap(), b))
    }

    pub fn _collides_iter(&self) -> impl Iterator<Item = (Entity, &Collider<f32, DefaultBodyHandle>)> {
        self.colliders.iter().map(|c| (*c.1.user_data().unwrap().downcast_ref::<Entity>().unwrap(), c.1))
    }
}

#[derive(Default, Debug)]
pub struct Inputs {
    pub pressed_keys: HashSet<input::keyboard::KeyCode>,
    pub mouse_clicked: HashSet<input::mouse::MouseButton>,
    pub mouse_pressed: HashSet<input::mouse::MouseButton>,
    pub mouse_pos: Point2f,
}

#[derive(Default, Debug)]
pub struct Settings {
    pub is_debug_info: bool,
    pub is_debug_targeting: bool,
    pub is_debug_physic: bool,
}

#[derive(Default, Debug)]
pub struct UiHub {
    pub menu: Menu,
    pub debug_window: DebugWindow,
}
impl<'a> UiBuilder<'a> for UiHub {
    type Data = (&'a Entity, &'a mut Settings, &'a mut SpawnQueue);

    fn build(
        &mut self,
        ui: &mut imgui::Ui,
        tex: &mut TextureProvider<'a>,
        (player, settings, spawn_queue): Self::Data,
    ) {
        self.menu.build(ui, tex, settings);

        if self.menu.is_show_spawn_window {
            self.debug_window.build(ui, tex, (player, spawn_queue));
        }
    }
}

#[derive(Default, Debug)]
pub struct SpawnQueue(pub Queue<SpawnItem>);

#[derive(Debug)]
pub enum SpawnItem {
    Entity(String, Point2f),
    Item(String, u32, Entity),
}
