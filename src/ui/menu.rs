use super::system::{UiBuilder, UiContext};
use crate::ecs::resource::UiData;
use imgui::*;

#[derive(Default, Debug)]
pub struct Menu {
    pub is_show_spawn_window: bool,
    pub is_show_arena_settings: bool,
    pub is_show_inventory: bool,
}
impl<'a> UiBuilder<&mut UiData<'a>> for Menu {
    fn build<'ctx>(&mut self, ui: &mut Ui, _: &mut UiContext<'ctx>, data: &mut UiData<'a>) {
        ui.main_menu_bar(|| {
            if ui.small_button(im_str!("Inventory")) {
                self.is_show_inventory = true;
            }

            ui.menu(im_str!("Debug"), true, || {
                if ui.small_button(im_str!("Spawn window")) {
                    self.is_show_spawn_window = !self.is_show_spawn_window;
                }
                if ui.small_button(im_str!("Arena settings")) {
                    self.is_show_arena_settings = !self.is_show_arena_settings;
                }
                ui.checkbox(im_str!("Render debug info"), &mut data.scene_controls.is_debug_info);
                ui.checkbox(im_str!("Render targeting"), &mut data.scene_controls.is_debug_targeting);
                ui.checkbox(im_str!("Render physic"), &mut data.scene_controls.is_debug_physic);
            });
        });
    }
}
