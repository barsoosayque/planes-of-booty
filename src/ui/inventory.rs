use super::system::{UiBuilder, UiContext};
use crate::{
    assets::*,
    ecs::{component::*, resource::UiData},
    within_group, within_window,
};
use imgui::*;
use specs::Entity;
use std::collections::BTreeSet as Set;

#[derive(Default, Debug)]
pub struct InventoryWindow {
    pub show_inventories_for: Set<Entity>,
}
impl InventoryWindow {
    const CELL: f32 = 50.0;
    const PAD: f32 = 10.0;
    const PCELL: f32 = Self::CELL + Self::PAD;
}
macro_rules! item_tooltip {
    ($item:expr, $ui:expr, $ctx:expr, $data:expr) => {
        if let Some(named) = $data.named.get($item) {
            $ui.bullet_text(&ImString::new(named.name));
            $ui.text(&named.description);
        }
        if let Some(weapon) = $data.weapons.get($item) {
            $ui.spacing();
            $ui.text_colored([0.73, 0.47, 0.38, 1.0], im_str!("Weapon stats:"));
            $ui.text(&ImString::new(&format!(
                "Damage: {} | Clip size: {} | Reloading time: {:.2} sec",
                weapon.damage, weapon.clip_size, weapon.reloading_time
            )));
        }
        if let Some(quality) = $data.qualities.get($item) {
            $ui.spacing();
            let color = match quality.rarity {
                Rarity::Common => [0.33, 0.33, 0.33, 1.0],
                Rarity::Rare => [0.06, 0.39, 0.53, 1.0],
                Rarity::Epic => [0.46, 0.16, 0.36, 1.0],
            };
            $ui.text_colored(color, &format!("Rarity: {}", quality.rarity));
        }
    };
}
macro_rules! item_box {
    ($item_box:expr, $pos:expr, $ui:expr, $ctx:expr, $data:expr) => {
        let w_pos = $ui.window_pos();
        let [w_x, w_y] = [w_pos[0] - $ui.scroll_x(), w_pos[1] - $ui.scroll_y()];
        $ui.set_cursor_pos($pos);
        let frame_asset = if $ui.is_mouse_hovering_rect(
            [$pos[0] + w_x, $pos[1] + w_y],
            [$pos[0] + Self::CELL + w_x, $pos[1] + Self::CELL + w_y]
        ) {
            $data.assets.get::<ImageAsset>("/sprites/ui/item-frame-hov.png", $ctx.as_mut()).unwrap()
        } else {
            $data.assets.get::<ImageAsset>("/sprites/ui/item-frame.png", $ctx.as_mut()).unwrap()
        };

        let frame = $ctx.get_texture_id_for(&frame_asset);
        Image::new(frame, [Self::CELL, Self::CELL]).build($ui);
        if $ui.is_item_hovered_with_flags(ItemHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM) {
        }
        $ui.set_cursor_pos($pos);

        if let Some((item, count)) = $item_box {
            let sprite = match $data.sprites.get(*item) {
                Some(Sprite { asset: SpriteAsset::Single { value }, .. }) => value,
                _ => {
                    log::warn!("There is no image for item {:?} ! (Shoulde be single asset in Sprite)", item);
                    return;
                },
            };

            within_group!($ui => {
                Image::new($ctx.get_texture_id_for(&sprite), [Self::CELL, Self::CELL]).build($ui);

                let count = ImString::from(format!("{}", count));
                let [text_width, text_height] = $ui.calc_text_size(&count, false, 0.0);
                $ui.set_cursor_pos([Self::CELL + $pos[0] - text_width, Self::CELL + $pos[1] - text_height]);
                $ui.text(&count);
            });

            if $ui.is_item_hovered_with_flags(ItemHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM) {
                // make that macro as well
                $ui.tooltip(|| {
                    item_tooltip!(*item, $ui, $ctx, $data);
                });
            }
        }
    };
}
macro_rules! items {
    ($inv:expr, $ui:expr, $ctx:expr, $data:expr) => {
        let per_row = (($ui.window_content_region_width() - Self::PAD) / (Self::PCELL)).ceil() as usize;
        for (i, item_box) in $inv.content.iter().enumerate() {
            let pos = [Self::PAD + Self::PCELL * (i % per_row) as f32, Self::PAD + Self::PCELL * (i / per_row) as f32];
            item_box!(&item_box, pos, $ui, $ctx, $data);
        }
    };
}
impl<'a> UiBuilder<&mut UiData<'a>> for InventoryWindow {
    fn build<'ctx>(&mut self, ui: &mut Ui, ctx: &mut UiContext<'ctx>, data: &mut UiData<'a>) {
        for e in &self.show_inventories_for {
            Window::new(im_str!("Inventory"))
                .position_pivot([0.5, 0.5])
                .resizable(false)
                .focus_on_appearing(true)
                .size([0.0, 0.0], Condition::Once)
                .build(ui, || {
                    if let Some(inventory) = data.inventories.get(*e) {
                        ui.bullet_text(im_str!("Content:"));
                        within_window!(ChildWindow::new("inv").size([380.0, 260.0]).border(true), &ui => {
                            if inventory.content.is_empty() {
                                let txt = im_str!("Empty !");
                                let [max_w, max_h] = ui.content_region_max();
                                let [text_w, text_h] = ui.calc_text_size(txt, false, 0.0);
                                ui.set_cursor_pos([(max_w - text_w) * 0.5, (max_h - text_h) * 0.5]);
                                ui.text(txt);
                            } else {
                                items!(&inventory, ui, ctx, data);
                            }
                        });
                    }
                    if let Some(weaponry) = data.weaponries.get(*e) {
                        ui.bullet_text(im_str!("Weapons:"));
                        within_window!(ChildWindow::new("weapon").size([380.0, 70.0]), &ui => {
                            let slots = [&weaponry.primary, &weaponry.secondary];
                            let [max_w, _] = ui.content_region_max();
                            for (i, item_box) in slots.iter().enumerate() {
                                let pos = [
                                    (Self::PCELL * i as f32) + (max_w - (Self::PCELL) * slots.len() as f32) * 0.5,
                                    Self::PAD
                                ];
                                item_box!(item_box, pos, ui, ctx, data);
                            }
                        });
                    }
                });
        }
    }
}
