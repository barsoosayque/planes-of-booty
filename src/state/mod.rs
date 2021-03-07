use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod game;
mod loading;
mod main_menu;

mod stage {
    pub const APP_STATE: &'static str = "app-state";
}

pub type States = bevy::ecs::State<State>;

#[derive(Clone)]
pub enum State {
    InitialLoading,
    MainMenu,
    Game,
}

pub struct StatePlugin;
impl Plugin for StatePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(EguiPlugin)
            .add_resource(bevy::ecs::State::new(State::InitialLoading))
            .add_stage_after(bevy::prelude::stage::UPDATE, stage::APP_STATE, StateStage::<State>::default())
            // Initial Loading
            .on_state_update(stage::APP_STATE, State::InitialLoading, loading::ui_system.system())
            .on_state_update(stage::APP_STATE, State::InitialLoading, loading::assets_watcher_system.system())
            // Main Menu
            .on_state_update(stage::APP_STATE, State::MainMenu, main_menu::ui_system.system())
            // Game
            .on_state_enter(stage::APP_STATE, State::Game, game::setup.system());
    }
}
