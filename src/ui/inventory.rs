use super::system::{UiBuilder, UiContext};
use crate::ecs::{component::*, resource::UiData};
use imgui::*;
use specs::Entity;
use std::collections::BTreeSet as Set;

#[derive(Default, Debug)]
pub struct InventoryWindow {
    pub show_inventories_for: Set<Entity>,
}
impl<'a> InventoryWindow {
    const CELL: f32 = 50.0;
    const PAD: f32 = 10.0;

    fn build_items<'ctx>(&self, inv: &Inventory, ui: &Ui, ctx: &mut UiContext<'ctx>, data: &UiData<'a>) {
        let per_row = ((ui.window_content_region_width() - Self::PAD) / (Self::CELL + Self::PAD)).ceil() as usize;
        for (i, (item, count)) in inv.content.iter().enumerate() {
            if let Some(Sprite { asset: SpriteAsset::Single { value }, .. }) = data.sprites.get(*item) {
                let pos = [
                    Self::PAD + (Self::CELL + Self::PAD) * (i % per_row) as f32,
                    Self::PAD + (Self::CELL + Self::PAD) * (i / per_row) as f32,
                ];
                ui.set_cursor_pos(pos);
                let grp = ui.begin_group();
                Image::new(ctx.get_texture_id_for(&value), [Self::CELL, Self::CELL])
                    .border_col([1.0, 1.0, 1.0, 0.3])
                    .build(ui);

                let count = ImString::from(format!("{}", count));
                let [text_width, text_height] = ui.calc_text_size(&count, false, 0.0);
                ui.set_cursor_pos([
                    Self::CELL + pos[0] - text_width,
                    Self::CELL + pos[1] - text_height,
                ]);
                ui.text(&count);
                grp.end(ui);

                if ui.is_item_hovered_with_flags(ItemHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM) {
                    ui.tooltip(|| {
                        if let Some(named) = data.named.get(*item) {
                            ui.bullet_text(&ImString::new(named.name));
                            ui.text(&named.description);
                        }
                        if let Some(quality) = data.qualities.get(*item) {
                            let color = match quality.rarity {
                                Rarity::Common => [0.33, 0.33, 0.33, 1.0],
                                Rarity::Rare => [0.06, 0.39, 0.53, 1.0],
                                Rarity::Epic => [0.46, 0.16, 0.36, 1.0],
                            };
                            ui.text_colored(color, &format!("Rarity: {}", quality.rarity));
                        }
                    });
                }
            } else {
                log::warn!("There is no image for item {:?} ! (Shoulde be single asset in Sprite)", item);
            }
        }
    }
}
impl<'a> UiBuilder<&mut UiData<'a>> for InventoryWindow {
    fn build<'ctx>(&mut self, ui: &mut Ui, ctx: &mut UiContext<'ctx>, data: &mut UiData<'a>) {
        for e in &self.show_inventories_for {
            if let Some(inventory) = data.inventories.get(*e) {
                Window::new(im_str!("Inventory"))
                    .position_pivot([0.5, 0.5])
                    .resizable(false)
                    .focus_on_appearing(true)
                    .size([0.0, 0.0], Condition::Once)
                    .build(ui, || {
                        ui.text(im_str!("Content:"));
                        ChildWindow::new("content").size([380.0, 380.0]).border(true).build(&ui, || {
                            if inventory.content.is_empty() {
                                ui.text(im_str!("Empty !"));
                            } else {
                                self.build_items(&inventory, ui, ctx, data);
                            }
                        });
                    });
            } else {
                log::warn!("No inventory to render for entity {:?}..", e);
                continue;
            }
        }
    }
}
