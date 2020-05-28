use super::{component::*, tag};
use crate::{
    map::{Generator, Chunk},
    assets::AssetManager,
    attack::{ProjectileBuilder, ProjectileDef},
    math::{Point2f, Vec2f},
    ui::*,
};
use ggez::{input, graphics};
use nphysics2d::{
    force_generator::DefaultForceGeneratorSet,
    joint::DefaultJointConstraintSet,
    object::{Collider, DefaultBodyHandle, DefaultBodySet, DefaultColliderSet, RigidBody, DefaultColliderHandle},
    world::{DefaultGeometricalWorld, DefaultMechanicalWorld},
};
use specs::prelude::*;
use std::collections::{HashSet, VecDeque as Queue};

#[derive(Default, Debug)]
pub struct DeltaTime(pub std::time::Duration);

#[derive(Default, Debug)]
pub struct Camera {
    pub pos: Vec2f,
    pub target: Option<Entity>,
    draw_params: graphics::DrawParam
}
impl Camera {
    pub fn apply(&mut self, ctx: &mut ggez::Context) {
        let win_size = graphics::window(ctx).get_inner_size().unwrap();
        self.draw_params = graphics::DrawParam::new()
            .dest(Point2f::new(-self.pos.x + win_size.width as f32 * 0.5, -self.pos.y + win_size.height as f32 * 0.5));
        graphics::push_transform(ctx, Some(self.draw_params.to_matrix()));
        graphics::apply_transformations(ctx).unwrap();
    }

    pub fn revert(&self, ctx: &mut ggez::Context) {
        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx).unwrap();
    }

    pub fn project(&self, v: &Point2f) -> Point2f {
         Point2f::new(v.x - self.draw_params.dest.x, v.y - self.draw_params.dest.y)
    }

    pub fn unproject(&self, v: &Point2f) -> Point2f {
         Point2f::new(v.x + self.draw_params.dest.x, v.y + self.draw_params.dest.y)
    }
}

#[derive(Default, Debug)]
pub struct ChunkMap {
    pub chunks: Vec<Chunk>,
    pub generator: Generator
}

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

    pub fn entity_for_collider(&self, handle: &DefaultColliderHandle) -> Option<&Entity> {
        self.colliders.get(*handle)
            .and_then(|c| c.user_data())
            .and_then(|d| d.downcast_ref::<Entity>())
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

#[derive(SystemData)]
pub struct UiData<'a> {
    pub entities: Entities<'a>,
    pub reflections: ReadStorage<'a, Reflection>,
    pub player_tag: ReadStorage<'a, tag::Player>,

    pub inventories: WriteStorage<'a, Inventory>,
    pub weaponries: WriteStorage<'a, Weaponry>,
    pub hpools: ReadStorage<'a, HealthPool>,

    pub wpn_props: ReadStorage<'a, WeaponProperties>,
    pub wpn_attacks: ReadStorage<'a, WeaponAttack>,
    pub named: ReadStorage<'a, Named>,
    pub qualities: ReadStorage<'a, Quality>,
    pub stacks: WriteStorage<'a, Stackable>,

    pub sprites: ReadStorage<'a, Sprite>,

    pub spawn_queue: Write<'a, SpawnQueue>,
    pub inputs: Write<'a, Inputs>,
    pub settings: Write<'a, Settings>,
    pub assets: Write<'a, AssetManager>,
    pub camera: Read<'a, Camera>,
}
#[derive(Default, Debug)]
pub struct UiHub {
    pub menu: Menu,
    pub hud: Hud,
    pub debug_window: DebugWindow,
    pub inventory_window: InventoryWindow,
}
impl<'a> UiBuilder<&mut UiData<'a>> for UiHub {
    fn build<'ctx>(&mut self, ui: &mut imgui::Ui, ctx: &mut UiContext<'ctx>, data: &mut UiData<'a>) {
        self.menu.build(ui, ctx, data);
        if self.menu.is_show_spawn_window {
            self.debug_window.build(ui, ctx, data);
        }
        self.inventory_window.build(ui, ctx, data);
        self.hud.build(ui, ctx, data);
    }
}

#[derive(Default, Debug)]
pub struct SpawnQueue(pub Queue<SpawnItem>);

impl ProjectileBuilder for SpawnQueue {
    fn build(&mut self, def: ProjectileDef) {
        self.0.push_back(SpawnItem::Projectile(def));
    }
}

#[derive(Debug)]
pub enum SpawnItem {
    Entity(String, Point2f),
    Item(String, u32, Entity),
    Projectile(ProjectileDef),
}
