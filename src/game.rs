use crate::assets::*;
use crate::ecs::resource::*;
use ggez::event::EventHandler;
use ggez::timer;
use ggez::{graphics, Context, GameResult};
use specs::prelude::*;

pub struct Game {
    world: World,
    assets: Assets,
}

impl Game {
    pub fn new(_ctx: &mut Context) -> Game {
        let mut world = World::new();
        let mut dispatcher = DispatcherBuilder::new().build();
        world.insert(DeltaTime(std::time::Duration::new(0, 0)));
        dispatcher.setup(&mut world);

        Game {
            world,
            assets: Assets::new(warmy::StoreOpt::default()).unwrap(),
        }
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // update delta
        let mut delta = self.world.write_resource::<DeltaTime>();
        delta.0 = timer::delta(ctx);

        self.assets.get::<ImageAsset>(&"sprites/ship-north.png".into(), ctx).unwrap();
        self.assets.sync(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::from_rgb_u32(0x7cd6d4));
        graphics::present(ctx)
    }
}
