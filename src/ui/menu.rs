use super::system::UiBuilder;
use imgui::*;

#[derive(Default, Debug)]
pub struct Menu {
    pub is_show_spawn_window: bool,
    pub is_debug_info: bool,
    pub is_debug_targeting: bool,
    pub is_debug_physic: bool,
}
impl UiBuilder for Menu {
    fn build(&mut self, ui: &mut imgui::Ui) {
        ui.main_menu_bar(|| {
            ui.menu(im_str!("Debug"), true, || {
                if ui.small_button(im_str!("Toggle spawn window")) {
                    self.is_show_spawn_window = !self.is_show_spawn_window
                }
                ui.checkbox(im_str!("Render debug info"), &mut self.is_debug_info);
                ui.checkbox(im_str!("Render targeting"), &mut self.is_debug_targeting);
                ui.checkbox(im_str!("Render physic"), &mut self.is_debug_physic);
            });
        });
    }
}
