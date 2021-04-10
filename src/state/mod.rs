use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod game;
mod loading;
mod main_menu;

pub type States = bevy::ecs::schedule::State<self::State>;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum State {
    InitialLoading,
    MainMenu,
    Game,
}

pub struct StatePlugin;
impl Plugin for StatePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_plugin(EguiPlugin)
        .add_plugin(loading::LoadingStatePlugin)
        .add_plugin(game::GameStatePlugin)
        .add_plugin(main_menu::MainMenuStatePlugin)
        .add_state(State::InitialLoading);
    }
}
