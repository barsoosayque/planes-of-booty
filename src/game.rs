use crate::assets::*;
use crate::ecs::{component::*, resource::*, system::*, tag};
use ggez::event::EventHandler;
use ggez::timer;
use ggez::{graphics, Context, GameResult};
use specs::prelude::*;

pub struct Game {
    world: World,
    dispatcher: Dispatcher<'static, 'static>,
    assets: AssetManager,
}

impl Game {
    pub fn new(ctx: &mut Context) -> Game {
        let mut world = World::new();
        let mut dispatcher = DispatcherBuilder::new()
            .with(MovementSystem, "movement_system", &[])
            .with(InputsSystem, "inputs_system", &[])
            .build();
        world.insert(DeltaTime(std::time::Duration::new(0, 0)));
        world.register::<tag::Player>();
        world.register::<Movement>();
        world.register::<Transform>();
        world.register::<DirectionalSprite>();
        dispatcher.setup(&mut world);

        let mut assets = AssetManager::new();

        crate::entity::player(&mut world, ctx, &mut assets);

        Game {
            world,
            dispatcher,
            assets,
        }
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        {
            // update delta time
            let mut delta = self.world.write_resource::<DeltaTime>();
            delta.0 = timer::delta(ctx);
        }
        {
            // update inputs
            let mut inputs = self.world.write_resource::<Inputs>();
            inputs.pressed_keys = ggez::input::keyboard::pressed_keys(ctx).to_owned();
        }

        self.dispatcher.dispatch(&self.world);
        self.world.maintain();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::from_rgb_u32(0x7cd6d4));
        SpriteRenderSystem(ctx).run_now(&self.world);
        graphics::present(ctx)
    }
}
