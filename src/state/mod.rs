use super::Config;
use bevy::prelude::*;

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
            .add_plugin(game::GamePlugin)
            .add_plugin(main_menu::MainMenuPlugin);
    }
}
