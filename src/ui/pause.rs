use super::system::{UiBuilder, UiContext};
use crate::{centered_text, ecs::resource::*};
use imgui::*;

#[derive(Default, Debug)]
pub struct PauseWindow {
    pub is_opened: bool,
    pub is_back: bool,
    pub is_restart: bool,
}
impl<'a> UiBuilder<&mut UiData<'a>> for PauseWindow {
    fn build<'ctx>(&mut self, ui: &mut Ui, _: &mut UiContext<'ctx>, _: &mut UiData<'a>) {
        if self.is_opened {
            ui.open_popup(im_str!("pause"))
        }

        let token = ui.push_style_colors(&[(StyleColor::ModalWindowDimBg, [0.0, 0.0, 0.0, 0.7])]);
        ui.popup_modal(im_str!("pause"))
            .title_bar(false)
            .movable(false)
            .resizable(false)
            .always_auto_resize(true)
            .build(|| {
                centered_text!(ui; "Pause menu"; width);
                ui.spacing();
                if ui.button(im_str!("Resume"), [300.0, 50.0]) {
                    ui.close_current_popup();
                    self.is_opened = false;
                }
                ui.spacing();
                if ui.button(im_str!("Restart"), [300.0, 50.0]) {
                    self.is_restart = true;
                }
                ui.spacing();
                if ui.button(im_str!("Exit"), [300.0, 50.0]) {
                    self.is_back = true
                }
            });
        token.pop(ui);
    }
}
