use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use crate::config::Config;

mod game;
mod main_menu;

mod stage {
    pub const APP_STATE: &'static str = "app-state";
}

pub type States = bevy::ecs::State<State>;

#[derive(Clone)]
pub enum State {
    MainMenu,
    Game,
}

pub struct StatePlugin;
impl Plugin for StatePlugin {
    fn build(&self, app: &mut AppBuilder) {
        let state = {
            let config = app.resources().get::<Config>().unwrap();
            bevy::ecs::State::new(if config.skip_menu { State::Game } else { State::MainMenu })
        };

        app.add_resource(state)
            .add_stage_after(bevy::prelude::stage::UPDATE, stage::APP_STATE, StateStage::<State>::default())
            .add_plugin(EguiPlugin)
            // Main menu
            .on_state_update(stage::APP_STATE, State::MainMenu, main_menu::ui_system.system())
            // Game
            .on_state_enter(stage::APP_STATE, State::Game, game::setup.system());
    }
}
