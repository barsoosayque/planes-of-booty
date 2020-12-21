use bevy::prelude::*;
use super::{stage, State::MainMenu};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.on_state_enter(stage::APP_STATE, MainMenu, Self::setup.system());
    }
}

impl MainMenuPlugin {
    fn setup() {
        debug!("hi from main menu");
    }
}
