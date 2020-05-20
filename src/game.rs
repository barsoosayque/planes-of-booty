use crate::assets::*;
use crate::math::*;
use crate::ecs::{component::*, resource::*, system::*, tag};
use crate::entity;
use crate::ui::{self, system::ImGuiSystem};
use ggez::event::EventHandler;
use ggez::timer;
use ggez::{graphics, Context, GameResult};
use specs::prelude::*;

pub struct Game {
    world: World,
    dispatcher: Dispatcher<'static, 'static>,
    assets: AssetManager,
    imgui: ImGuiSystem,
}

impl Game {
    pub fn new(ctx: &mut Context) -> Game {
        let mut world = World::new();
        let mut dispatcher = DispatcherBuilder::new()
            .with(SearchForTargetSystem, "search_for_target_system", &[])
            .with(FollowTargetSystem, "follow_target_system", &[])
            .with(MovementSystem, "movement_system", &[])
            .with(InputsSystem, "inputs_system", &[])
            .build();
        world.insert(DeltaTime(std::time::Duration::new(0, 0)));
        world.register::<tag::Player>();
        world.register::<Movement>();
        world.register::<Transform>();
        world.register::<Sprite>();
        world.register::<DirectionalSprite>();
        world.register::<Target>();
        world.register::<SearchForTarget>();
        world.register::<FollowTarget>();
        world.register::<Faction>();

        dispatcher.setup(&mut world);

        let mut assets = AssetManager::new();
        let imgui = ImGuiSystem::new(ctx);

        let player = entity::spawn_player(&mut world, ctx, &mut assets);
        world.write_storage::<Transform>().insert(
            player,
            Transform {
                pos: Vec2f::new(300.0, 300.0),
                ..Transform::default()
            },
        ).unwrap();

        entity::spawn_pirate_raft(&mut world, ctx, &mut assets);

        Game {
            world,
            dispatcher,
            assets,
            imgui,
        }
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let delta = {
            // update delta time
            let mut delta = self.world.write_resource::<DeltaTime>();
            delta.0 = timer::delta(ctx);
            delta.0
        };

        {
            // update inputs
            let mut inputs = self.world.write_resource::<Inputs>();
            inputs.pressed_keys = ggez::input::keyboard::pressed_keys(ctx).to_owned();
        }

        self.dispatcher.dispatch(&self.world);
        self.world.maintain();
        self.imgui.update(ctx, delta);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::from_rgb_u32(0x7cd6d4));
        SpriteRenderSystem(ctx).run_now(&self.world);
        DebugRenderSystem(ctx).run_now(&self.world);
        self.imgui.render(ctx, vec![ui::debug::DebugToolsUi]);
        graphics::present(ctx)
    }
}
