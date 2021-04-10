use super::{State, States};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use egui::CentralPanel;

pub struct MainMenuStatePlugin;

impl Plugin for MainMenuStatePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(SystemSet::on_update(State::MainMenu).with_system(ui_system.system()));
    }
}

fn ui_system(egui: ResMut<EguiContext>, mut states: ResMut<States>) {
    let ctx = egui.ctx();

    CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.label(&format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")));

            if ui.button("play").clicked() {
                states.set(State::Game).unwrap();
            }
        });
    });
}
