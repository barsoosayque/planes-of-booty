use crate::{render::PipelineStatus, Config};

use super::{State, States};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use egui::CentralPanel;

pub struct LoadingStatePlugin;

impl Plugin for LoadingStatePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_system_set(
                SystemSet::on_update(State::InitialLoading)
                    .with_system(ui_system.system())
                    .with_system(assets_watcher_system.system()),
            );
    }
}

fn ui_system(egui: ResMut<EguiContext>) {
    let ctx = egui.ctx();

    CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.label("Loading...");
        });
    });
}

fn assets_watcher_system(
    config: Res<Config>,
    pipeline_status: Res<PipelineStatus>,
    mut states: ResMut<States>,
) {
    if pipeline_status.is_ready() {
        let state = {
            if config.skip_menu {
                State::Game
            } else {
                State::MainMenu
            }
        };

        states.set(state).unwrap();
        info!("Everything is loaded.")
    }
}
