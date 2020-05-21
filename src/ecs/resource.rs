use crate::math::{Point2f, Vec2f};
use crate::ui::*;
use ggez::input;
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::object::{DefaultBodySet, DefaultColliderSet};
use nphysics2d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};
use std::collections::HashSet;
use std::collections::VecDeque as Queue;

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
}

#[derive(Default, Debug)]
pub struct Inputs {
    pub pressed_keys: HashSet<input::keyboard::KeyCode>,
    pub mouse_clicked: HashSet<input::mouse::MouseButton>,
    pub mouse_pressed: HashSet<input::mouse::MouseButton>,
    pub mouse_pos: Point2f,
}

#[derive(Default, Debug)]
pub struct UiHub {
    pub menu: Menu,
    pub debug_window: DebugWindow,
}
impl UiBuilder for UiHub {
    fn build(&mut self, ui: &mut imgui::Ui) {
        self.menu.build(ui);

        if self.menu.is_show_spawn_window {
            self.debug_window.build(ui);
        }
    }
}

#[derive(Default, Debug)]
pub struct SpawnQueue(pub Queue<SpawnItem>);
#[derive(Default, Debug)]
pub struct SpawnItem {
    pub id: String,
    pub pos: Point2f,
}
