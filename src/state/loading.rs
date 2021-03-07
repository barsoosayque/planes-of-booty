use crate::{render::PipelineStatus, Config};

use super::{State, States};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use egui::CentralPanel;

pub fn ui_system(mut egui: ResMut<EguiContext>) {
    let ctx = &mut egui.ctx;

    CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.label("Loading...");
        });
    });
}

pub fn assets_watcher_system(config: Res<Config>, pipeline_status: Res<PipelineStatus>, mut states: ResMut<States>) {
    if pipeline_status.is_ready() {
        let state = {
            if config.skip_menu {
                State::Game
            } else {
                State::MainMenu
            }
        };

        states.set_next(state).unwrap();
    }
}
