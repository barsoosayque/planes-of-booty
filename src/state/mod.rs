use bevy::prelude::*;

mod main_menu;
mod game;

mod stage {
    pub const APP_STATE: &'static str = "app-state";
}

pub type States = bevy::ecs::State<State>;

#[derive(Clone)]
pub enum State {
    MainMenu,
    Game
}

pub struct StatePlugin;
impl Plugin for StatePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(bevy::ecs::State::new(State::MainMenu))
            .add_stage_after(bevy::prelude::stage::UPDATE, stage::APP_STATE, StateStage::<State>::default())
            .add_plugin(game::GamePlugin)
            .add_plugin(main_menu::MainMenuPlugin);
    }
}
