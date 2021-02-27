use super::{State, States};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use egui::CentralPanel;

pub fn ui_system(mut egui: ResMut<EguiContext>, mut states: ResMut<States>) {
    let ctx = &mut egui.ctx;

    CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.label(&format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")));

            if ui.button("play").clicked {
                states.set_next(State::Game).unwrap();
            }
        });
    });
}
