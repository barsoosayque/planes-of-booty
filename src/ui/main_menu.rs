use super::system::{UiBuilder, UiContext};
use crate::assets::*;
use crate::{centered_text, within_window};
use imgui::*;

#[derive(Default, Debug)]
pub struct MainMenu {
    pub is_play: bool,
    pub is_how_to_play: bool,
    pub is_exit: bool
}
impl<'a> UiBuilder<&mut AssetManager> for MainMenu {
    fn build<'ctx>(&mut self, ui: &mut Ui, ctx: &mut UiContext<'ctx>, assets: &mut AssetManager) {
        within_window!(Window::new(im_str!("MainMenu"))
            .position([0.0, 0.0], Condition::Always)
            .position_pivot([0.0, 0.0])
            .resizable(false)
            .movable(false)
            .collapsible(false)
            .title_bar(false)
            .draw_background(false)
            .focus_on_appearing(false)
            .size(ui.io().display_size, Condition::Always), &ui => {
                let [ww, wh] = ui.window_size();

                ui.set_cursor_pos([(ww - 430.0) * 0.5, wh * 0.1]);
                let logo = assets.get::<ImageAsset>("/sprites/ui/logo.png", ctx.as_mut()).unwrap();
                Image::new(ctx.get_texture_id_for(&logo), [430.0, 190.0]).build(ui);

                ui.set_cursor_pos([(ww - 300.0) * 0.5, ui.cursor_pos()[1] + 30.0]);
                self.is_play = ui.button(im_str!("Play"), [300.0, 50.0]);

                ui.set_cursor_pos([(ww - 300.0) * 0.5, ui.cursor_pos()[1] + 30.0]);
                if ui.button(im_str!("How to play"), [300.0, 50.0]) {
                    self.is_how_to_play = true;
                }

                ui.set_cursor_pos([(ww - 300.0) * 0.5, ui.cursor_pos()[1] + 30.0]);
                self.is_exit = ui.button(im_str!("Exit"), [300.0, 50.0]);

                ui.set_cursor_pos([0.0, wh - 50.0]);
                centered_text!(ui; format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")); width);
        });

        if self.is_how_to_play {
            within_window!(Window::new(im_str!("How to play"))
                .resizable(false)
                .focus_on_appearing(true)
                .opened(&mut self.is_how_to_play)
                .size([500.0, 0.0], Condition::Once), ui => {
                    ui.text_wrapped(im_str!("Planes of Booty quick tutorial."));
                    ui.spacing();
                    ui.bullet_text(im_str!("Gameplay"));
                    ui.text_wrapped(im_str!("\
                    Shoot all the eneimes down to clear the arena and earn some booty. \
                    Jump into the magic swirl to proceed to the next arena where a new pack \
                    of enemies will wait for you. Use fruits power ups in a dire moment to \
                    really turn the tide of the battle. \
                    "));
                    ui.spacing();
                    ui.bullet_text(im_str!("Controls"));
                    ui.text_wrapped(im_str!("\
                    [W] -- Move up\n\
                    [A] -- Move left\n\
                    [S] -- Move down\n\
                    [D] -- Move right\n\
                    [E] -- Interact (open chest or use swirl to chage level)\n\
                    [I] -- Open inventory\n\
                    [Mouse wheel] -- Change primary/secondary weapon\n\
                    [Mouse left button] -- Shoot your primary gun\n\
                    "));
            });
        }
    }
}
