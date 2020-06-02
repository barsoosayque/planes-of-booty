use super::system::{UiBuilder, UiContext};
use crate::{
    centered_text,
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
                .size([90.0, 125.0], Condition::Always), &ui => {
                    ui.set_cursor_pos([15.0, 10.0]);
                    let hp_base = data.assets.get::<ImageAsset>("/sprites/ui/hp-base.png", ctx.as_mut()).unwrap();
                    let hp_fill = data.assets.get::<ImageAsset>("/sprites/ui/hp-fill.png", ctx.as_mut()).unwrap();

                    let pos = ui.cursor_pos();
                    Image::new(ctx.get_texture_id_for(&hp_base), [60.0, 60.0]).build(ui);

                    let hp_lack = 1.0 - (hpool.hp as f32 / hpool.max_hp as f32);
                    ui.set_cursor_pos([pos[0], pos[1] + 60.0 * hp_lack]);
                    Image::new(ctx.get_texture_id_for(&hp_fill), [60.0, 60.0 * (1.0 - hp_lack)])
                        .uv0([0.0, hp_lack])
                        .build(ui);

                    centered_text!(ui; format!("Health:\n{} / {}", hpool.hp, hpool.max_hp); width);
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
                .size([90.0, 125.0], Condition::Always), &ui => {
                    if let Some(weapon) = weaponry.primary {
                        if let Some(Sprite{ asset: SpriteAsset::Single { value }, ..}) = data.sprites.get(weapon) {
                            ui.set_cursor_pos([15.0, 10.0]);
                            Image::new(ctx.get_texture_id_for(&value), [60.0, 60.0]).build(ui);
                        }
                        if let Some(prop) = data.wpn_props.get(weapon) {
                            centered_text!(ui; format!("Clip:\n{} / {}", prop.clip, prop.clip_size); width);
                            if prop.reloading > 0.0 {
                                centered_text!(ui; "> Reload"; width);
                            } else if prop.cooldown > 0.0 {
                                centered_text!(ui; format!("> {:.0}%", (1.0 - prop.cooldown / prop.cooldown_time) * 100.0); width);
                            } else {
                                centered_text!(ui; "> Ready"; width);
                            }
                        }
                    } else {
                        centered_text!(ui; "Nothing\nEquiped"; width, height);
                    }
            });
        }

        if let Some((hotbar, _)) = (&data.hotbars, &data.player_tag).join().next() {
            within_window!(Window::new(im_str!("Hotbar"))
                .position([ui.io().display_size[0] * 0.5, ui.io().display_size[1]], Condition::Always)
                .position_pivot([0.5, 1.0])
                .resizable(false)
                .movable(false)
                .collapsible(false)
                .title_bar(false)
                .focus_on_appearing(false)
                .size([0.0, 0.0], Condition::Always), &ui => {
                    for (i, item_box) in hotbar.content.iter().enumerate() {
                        let [x, y] = ui.cursor_start_pos();
                        ui.set_cursor_pos([x + i as f32 * 70.0, y]);
                        let frame = data.assets.get::<ImageAsset>("/sprites/ui/item-frame.png", ctx.as_mut()).unwrap();
                        let pos = ui.cursor_pos();
                        Image::new(ctx.get_texture_id_for(&frame), [50.0, 50.0]).build(ui);
                        if let Some(Sprite{ asset: SpriteAsset::Single { value }, ..}) = item_box.and_then(|i| data.sprites.get(i)) {
                            ui.set_cursor_pos(pos);
                            Image::new(ctx.get_texture_id_for(&value), [50.0, 50.0]).build(ui);
                        }
                        ui.set_cursor_pos([pos[0] + 17.0, 45.0]);
                        ui.text(&format!("[{}]", i + 1));
                    }
            });
        }

        if let Some((consumer, _)) = (&data.consumers, &data.player_tag).join().next() {
            within_window!(Window::new(im_str!("Buffs"))
                .position([ui.io().display_size[0] * 0.5, ui.io().display_size[1] - 70.0], Condition::Always)
                .position_pivot([0.5, 1.0])
                .resizable(false)
                .draw_background(false)
                .movable(false)
                .collapsible(false)
                .title_bar(false)
                .scroll_bar(false)
                .focus_on_appearing(false)
                .size([0.0, 40.0], Condition::Always), &ui => {
                    for handle in &consumer.handles {
                        if let Some(icon) = handle.behaviour.icon(ctx.as_mut(), &mut data.assets) {
                            Image::new(ctx.get_texture_id_for(&icon), [30.0, 30.0]).build(ui);
                            ui.same_line(0.0);
                        }
                    }
            });
        }
    }
}
