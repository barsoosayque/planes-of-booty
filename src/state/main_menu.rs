use super::*;
use anyhow::Result;
use bevy_fallable::fallable_system;

pub struct MainMenuPlugin;

pub struct MainMenuComponent;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut AppBuilder) { app.add_system_to_stage(stage::UPDATE, Self::create_system.system()); }
}

impl MainMenuPlugin {
    fn create_system(added: Query<Added<MainMenuComponent>>) {
        for _ in added.iter() {
        }
    }
}
