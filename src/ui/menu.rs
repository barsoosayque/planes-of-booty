use super::system::{UiBuilder, UiContext};
use crate::ecs::resource::UiData;
use imgui::*;

#[derive(Default, Debug)]
pub struct Menu {
    pub is_show_spawn_window: bool,
    pub is_show_inventory: bool,
}
impl<'a> UiBuilder<&mut UiData<'a>> for Menu {
    fn build<'ctx>(&mut self, ui: &mut Ui, _: &mut UiContext<'ctx>, data: &mut UiData<'a>) {
        ui.main_menu_bar(|| {
            if ui.small_button(im_str!("Inventory")) {
                self.is_show_inventory = true;
            }

            ui.menu(im_str!("Debug"), true, || {
                if ui.small_button(im_str!("Toggle spawn window")) {
                    self.is_show_spawn_window = !self.is_show_spawn_window
                }
                ui.checkbox(im_str!("Render debug info"), &mut data.settings.is_debug_info);
                ui.checkbox(im_str!("Render targeting"), &mut data.settings.is_debug_targeting);
                ui.checkbox(im_str!("Render physic"), &mut data.settings.is_debug_physic);
            });
        });
    }
}
