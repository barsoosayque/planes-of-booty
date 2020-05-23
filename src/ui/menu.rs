use super::system::UiBuilder;
use crate::ecs::resource::Settings;
use imgui::*;

#[derive(Default, Debug)]
pub struct Menu {
    pub is_show_spawn_window: bool,
    pub is_show_inventory: bool,
}
impl<'a> UiBuilder<'a> for Menu {
    type Data = &'a mut Settings;

    fn build(&mut self, ui: &mut imgui::Ui, settings: Self::Data) {
        ui.main_menu_bar(|| {
            if ui.small_button(im_str!("Inventory")) {
                self.is_show_inventory = true;
            }

            ui.menu(im_str!("Debug"), true, || {
                if ui.small_button(im_str!("Toggle spawn window")) {
                    self.is_show_spawn_window = !self.is_show_spawn_window
                }
                ui.checkbox(im_str!("Render debug info"), &mut settings.is_debug_info);
                ui.checkbox(im_str!("Render targeting"), &mut settings.is_debug_targeting);
                ui.checkbox(im_str!("Render physic"), &mut settings.is_debug_physic);
            });
        });
    }
}
