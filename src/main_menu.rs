use crate::{
    game::Game,
    assets::AssetManager,
    scene::{Scene, SceneCommand},
    ui::{self, ImGuiSystem},
};
use ggez::{event::EventHandler, graphics, timer, Context, GameResult};

pub struct MainMenu {
    ui: ui::MainMenu,
    assets: AssetManager,
    imgui: ImGuiSystem,
}

impl MainMenu {
    pub fn new(ctx: &mut Context) -> Self {
        Self { ui: ui::MainMenu::default(), assets: AssetManager::default(), imgui: ImGuiSystem::new(ctx) }
    }
}

impl Scene for MainMenu {
    fn next_command(&self) -> Option<SceneCommand> { 
        if self.ui.is_play {
            Some(SceneCommand::ReplaceAll(|ctx| Box::new(Game::new(ctx))))
        } else {
            None
        }
    }

    fn draw_prev(&self) -> bool { false }
}
impl EventHandler for MainMenu {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.imgui.update(ctx, timer::delta(ctx), &mut self.ui, &mut self.assets);
        if self.ui.is_exit {
            ggez::event::quit(ctx);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::from_rgb_u32(0x151515));
        self.imgui.render(ctx);
        graphics::present(ctx)
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, width, height)).unwrap();
    }
}
