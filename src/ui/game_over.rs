use super::system::{UiBuilder, UiContext};
use crate::{assets::ImageAsset, ecs::resource::*};
use imgui::*;

#[derive(Default, Debug)]
pub struct GameOverWindow {
    pub is_opened: bool,
}
impl<'a> UiBuilder<&mut UiData<'a>> for GameOverWindow {
    fn build<'ctx>(&mut self, ui: &mut Ui, ctx: &mut UiContext<'ctx>, data: &mut UiData<'a>) {
        if self.is_opened {
            ui.open_popup(im_str!("game_over"))
        }

        let token = ui.push_style_colors(&[(StyleColor::ModalWindowDimBg, [0.0, 0.0, 0.0, 0.7])]);
        ui.popup_modal(im_str!("game_over"))
            .flags(WindowFlags::NO_BACKGROUND | WindowFlags::NO_DECORATION)
            .movable(false)
            .always_auto_resize(true)
            .build(|| {
                ui.set_cursor_pos([80.0, 0.0]);
                let game_over = data.assets.get::<ImageAsset>("/sprites/ui/game-over.png", ctx.as_mut()).unwrap();
                Image::new(ctx.get_texture_id_for(&game_over), [540.0, 380.0]).build(ui);
                ui.dummy([0.0,100.0]);

                if ui.button(im_str!("Restart"), [300.0, 50.0]) {
                    data.scene_controls.queue_restart = true;
                }
                ui.same_line(400.0);
                if ui.button(im_str!("Exit"), [300.0, 50.0]) {
                    data.scene_controls.queue_exit = true;
                }
            });
        token.pop(ui);
    }
}
