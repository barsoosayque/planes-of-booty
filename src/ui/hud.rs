use super::system::{UiBuilder, UiContext};
use crate::{
    assets::*,
    ecs::{component::*, resource::UiData},
    within_window,
};
use imgui::*;
use specs::Join;

#[derive(Default, Debug)]
pub struct Hud;
impl<'a> UiBuilder<&mut UiData<'a>> for Hud {
    fn build<'ctx>(&mut self, ui: &mut Ui, ctx: &mut UiContext<'ctx>, data: &mut UiData<'a>) {
        if let Some((hpool, _)) = (&data.hpools, &data.player_tag).join().next() {
            within_window!(Window::new(im_str!("HealthPool"))
                .position([0.0, ui.io().display_size[1]], Condition::Always)
                .position_pivot([0.0, 1.0])
                .resizable(false)
                .movable(false)
                .collapsible(false)
                .title_bar(false)
                .focus_on_appearing(false)
                .size([0.0, 0.0], Condition::Always), &ui => {
                    let hp_base = data.assets.get::<ImageAsset>("/sprites/ui/hp-base.png", ctx.as_mut()).unwrap();
                    let hp_fill = data.assets.get::<ImageAsset>("/sprites/ui/hp-fill.png", ctx.as_mut()).unwrap();

                    let pos = ui.cursor_start_pos();
                    Image::new(ctx.get_texture_id_for(&hp_base), [60.0, 60.0]).build(ui);

                    let hp_lack = 1.0 - (hpool.hp as f32 / hpool.max_hp as f32);
                    ui.set_cursor_pos([pos[0], pos[1] + 60.0 * hp_lack]);
                    Image::new(ctx.get_texture_id_for(&hp_fill), [60.0, 60.0 * (1.0 - hp_lack)])
                        .uv0([0.0, hp_lack])
                        .build(ui);

                    ui.text(format!("Health:\n{} / {}", hpool.hp, hpool.max_hp));
            });
        }

        if let Some((weaponry, _)) = (&data.weaponries, &data.player_tag).join().next() {
            within_window!(Window::new(im_str!("Ammo"))
                .position(ui.io().display_size, Condition::Always)
                .position_pivot([1.0, 1.0])
                .resizable(false)
                .movable(false)
                .collapsible(false)
                .title_bar(false)
                .focus_on_appearing(false)
                .size([0.0, 0.0], Condition::Always), &ui => {
                    if let Some(Sprite{ asset: SpriteAsset::Single { value }, ..}) = 
                        weaponry.primary.and_then(|e| data.sprites.get(e)) 
                    {
                        Image::new(ctx.get_texture_id_for(&value), [60.0, 60.0]).build(ui);
                    }
                    if let Some(prop) = weaponry.primary.and_then(|e| data.wpn_props.get(e)) {
                        ui.text(format!("Clip:\n{} / {}", prop.clip, prop.clip_size));
                        if prop.reloading > 0.0 {
                            ui.text("> Reload");
                        } else if prop.cooldown > 0.0 {
                            ui.text(format!("> {:.0}%", (1.0 - prop.cooldown / prop.cooldown_time) * 100.0));
                        } else {
                            ui.text("> Ready");
                        }
                    }
            });
        }
    }
}
