use crate::assets::*;
use crate::ecs::{component::*, resource::*, system::*, tag};
use crate::entity;
use crate::math::*;
use crate::ui::ImGuiSystem;
use ggez::event::EventHandler;
use ggez::timer;
use ggez::{graphics, Context, GameResult};
use specs::prelude::*;

pub struct Game {
    world: World,
    dispatcher: Dispatcher<'static, 'static>,
    imgui: ImGuiSystem,
}

impl Game {
    fn prespawn(&mut self) {
        let queue = &mut self.world.write_resource::<SpawnQueue>().0;
        queue.push_back(SpawnItem {
            id: "player".into(),
            pos: Point2f::new(300.0, 300.0),
        });
    }

    pub fn new(ctx: &mut Context) -> Self {
        let imgui = ImGuiSystem::new(ctx);
        let mut world = World::new();
        let mut dispatcher = DispatcherBuilder::new()
            .with(SearchForTargetSystem, "search_for_target_system", &[])
            .with(FollowTargetSystem, "follow_target_system", &[])
            .with(MovementSystem, "movement_system", &[])
            .with(InputsSystem, "inputs_system", &[])
            .with(PhysicSystem, "physic_system", &[])
            .build();
        world.insert(DeltaTime(std::time::Duration::new(0, 0)));
        world.insert(UiHub::default());
        world.insert(SpawnQueue::default());
        world.insert(AssetManager::default());
        world.insert(PhysicWorld::new(Vec2f::new(0.0, 0.0)));
        world.register::<tag::Player>();
        world.register::<Movement>();
        world.register::<Transform>();
        world.register::<Sprite>();
        world.register::<Target>();
        world.register::<SearchForTarget>();
        world.register::<FollowTarget>();
        world.register::<Faction>();
        world.register::<Physic>();
        dispatcher.setup(&mut world);

        let mut game = Self {
            world,
            dispatcher,
            imgui,
        };
        game.prespawn();
        game
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        {
            // update delta time
            let mut delta = self.world.write_resource::<DeltaTime>();
            delta.0 = timer::delta(ctx);
        };

        {
            // update inputs
            use ggez::input::{keyboard, mouse};
            let mut inputs = self.world.write_resource::<Inputs>();
            inputs.pressed_keys = keyboard::pressed_keys(ctx).to_owned();
            inputs.mouse_pos = Point2f::from(mouse::position(ctx));
            let new_press: std::collections::HashSet<mouse::MouseButton> =
                [mouse::MouseButton::Left, mouse::MouseButton::Right]
                    .iter()
                    .cloned()
                    .filter(|btn| mouse::button_pressed(ctx, *btn))
                    .collect();
            inputs.mouse_clicked = inputs
                .mouse_pressed
                .difference(&new_press)
                .cloned()
                .collect();
            inputs.mouse_pressed = new_press;
        }

        // run ui system before any other system so it can
        // consume input events
        UiSystem(ctx, &mut self.imgui).run_now(&self.world);
        self.dispatcher.dispatch(&self.world);

        // Systems can spawn entities using SpawnQueue resource
        for item in self.world.write_resource::<SpawnQueue>().0.drain(..) {
            let e = entity::spawn(&item.id, &self.world, ctx);
            let transform = Transform {
                pos: item.pos.to_vector(),
                ..Transform::default()
            };
            self.world
                .write_storage::<Transform>()
                .insert(e, transform)
                .unwrap();
        }

        self.world.maintain();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let ui_hub = self.world.read_resource::<UiHub>();
        graphics::clear(ctx, graphics::Color::from_rgb_u32(0x7cd6d4));
        if ui_hub.menu.is_debug_info {
            DebugInfoRenderSystem(ctx).run_now(&self.world);
        }
        if ui_hub.menu.is_debug_targeting {
            DebugTargetRenderSystem(ctx).run_now(&self.world);
        }
        SpriteRenderSystem(ctx).run_now(&self.world);
        if ui_hub.menu.is_debug_physic {
            DebugPhysicRenderSystem(ctx).run_now(&self.world);
        }
        UiRenderSystem(ctx, &mut self.imgui).run_now(&self.world);
        graphics::present(ctx)
    }
}
