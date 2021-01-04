use super::{stage, State};
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.on_state_enter(stage::APP_STATE, State::Game, Self::setup.system()).on_state_exit(
            stage::APP_STATE,
            State::Game,
            Self::destruct.system(),
        );
    }
}

impl GamePlugin {
    fn setup() {
        info!("Game: initialized");
    }

    fn destruct() {
        info!("Game: destructed");
    }
}
