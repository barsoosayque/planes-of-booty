use super::system::{UiBuilder, UiContext};
use crate::{
    assets::*,
    ecs::{component::*, resource::UiData},
    styled, within_group, within_tooltip, within_window,
};
use imgui::*;
use specs::prelude::*;
use std::collections::BTreeSet as Set;

// WARNING:
// Please, run from this file as far as you can.
// Run and never look back. Never return.
// You have been warned.

// Non owning view of ItemBox
#[derive(Debug)]
struct ItemBoxView {
    pub view: *mut ItemBox,
}
// It is ok to use pointer in ui because ui logic is purely
// single-threaded and always will
unsafe impl Send for ItemBoxView {}
unsafe impl Sync for ItemBoxView {}

#[derive(Debug)]
pub struct InventoryWindow {
    pub show_inventories_for: Set<Entity>,
    dragging_item: ItemBoxView,
}
impl Default for InventoryWindow {
    fn default() -> Self {
        Self { show_inventories_for: Set::new(), dragging_item: ItemBoxView { view: std::ptr::null_mut() } }
    }
}
impl InventoryWindow {
    const CELL: f32 = 50.0;
    const PAD: f32 = 10.0;
    const PCELL: f32 = Self::CELL + Self::PAD;

    pub fn dragging_item(&self) -> &ItemBox {
        unsafe {
            match self.dragging_item.view.as_ref() {
                Some(item) => &item,
                None => &None,
            }
        }
    }

    pub fn reset_dragging(&mut self) { self.dragging_item.view = std::ptr::null_mut(); }
}
macro_rules! drag_and_drop {
    ($item_box_to:expr, $item_box_from:expr, $data:expr) => {
        if let Some(mut viewed) = unsafe { $item_box_from.view.as_mut() } {
            if let (Some(to), Some(from)) = ($item_box_to, &mut viewed) {
                let (id_to, id_from) =
                    ($data.reflections.get(*to).unwrap().id, $data.reflections.get(*from).unwrap().id);

                if id_to == id_from && from != to {
                    let mut current = $data.stacks.get(*from).map(|s| s.current).unwrap_or(1);
                    if let Some(mut stack_to) = $data.stacks.get_mut(*to) {
                        let transfer_count = current.min(stack_to.stack_size - stack_to.current);
                        stack_to.current += transfer_count;
                        current = current.saturating_sub(transfer_count);
                    }
                    $data.stacks.get_mut(*from).unwrap().current = current;
                    if current == 0 {
                        $data.entities.delete(viewed.take().unwrap()).unwrap();
                        $item_box_from.view = std::ptr::null_mut();
                    }
                }
            } else {
                $item_box_to.replace(viewed.take().unwrap());
                $item_box_from.view = std::ptr::null_mut();
            }
        }
    };
}

// TODO: somehow bypass macro hygiene with self, ui, ctx and data
macro_rules! item_tooltip {
    ($self:expr, $item:expr, $ui:expr, $ctx:expr, $data:expr) => {
        let token = $ui.push_text_wrap_pos(400.0);
        if let Some(named) = $data.named.get($item) {
            $ui.bullet_text(&ImString::new(named.name));
            $ui.text(&named.description);
        }
        if let (Some(attack), Some(props)) = ($data.wpn_attacks.get($item), $data.wpn_props.get($item)) {
            $ui.separator();
            $ui.text_colored([0.78, 0.23, 0.20, 1.0], im_str!("It's a weapon:"));
            $ui.text(format!("* Damage: {}", props.damage));
            $ui.text(format!("* Accuracy: {:.0}%", (props.accuracy * 100.0).floor()));
            $ui.text(format!("* Clip size: {}", props.clip_size));
            $ui.text(format!("* Reloading time: {:.2}", props.reloading_time));
            $ui.text(format!("* Cooling speed: {:.2}", props.cooldown_time));
            if props.passive_reloading {
                $ui.text("* Can reload passively !");
            }
            $ui.text(&ImString::new(attack.pattern.description()));
        }
        if let Some(consumable) = $data.consumables.get($item) {
            $ui.separator();
            $ui.text_colored([0.81, 0.48, 0.72, 1.0], im_str!("It's a consumable:"));
            $ui.text(&ImString::new(consumable.behaviour.description()));
        }
        if let Some(quality) = $data.qualities.get($item) {
            $ui.separator();
            let color = match quality.rarity {
                Rarity::Common => [0.4, 0.4, 0.4, 1.0],
                Rarity::Rare => [0.0, 0.48, 1.0, 1.0],
                Rarity::Legendary => [1.0, 0.9, 0.36, 1.0],
            };
            $ui.text_colored(color, &format!("Rarity: {}", quality.rarity));
        }
        token.pop($ui);
    };
}
// Returns true if there is drag and drop for this box
macro_rules! item_box {
    ($self:expr, $item_box:expr, $pos:expr, $ui:expr, $ctx:expr, $data:expr) => {{
        $ui.set_cursor_pos($pos);
        let w_pos = $ui.window_pos();
        let [w_x, w_y] = [w_pos[0] - $ui.scroll_x(), w_pos[1] - $ui.scroll_y()];
        let frame_asset = if $ui.is_mouse_hovering_rect(
            [$pos[0] + w_x, $pos[1] + w_y],
            [$pos[0] + Self::CELL + w_x, $pos[1] + Self::CELL + w_y]
        ) && ($item_box.is_some() || $self.dragging_item().is_some()) {
            $data.assets.get::<ImageAsset>("/sprites/ui/item-frame-hov.png", $ctx.as_mut()).unwrap()
        } else {
            $data.assets.get::<ImageAsset>("/sprites/ui/item-frame.png", $ctx.as_mut()).unwrap()
        };

        let frame = $ctx.get_texture_id_for(&frame_asset);
        Image::new(frame, [Self::CELL, Self::CELL]).build($ui);
        $ui.set_cursor_pos($pos);

        if let Some(item) = $item_box {
            let sprite = match $data.sprites.get(*item) {
                Some(Sprite { asset: SpriteAsset::Single { value }, .. }) => value,
                _ => {
                    log::warn!("There is no image for item {:?} ! (Shoulde be single asset in Sprite)", item);
                    break;
                },
            };

            within_group!($ui => {
                let image_alpha = if $self.dragging_item().map(|b| b == *item).unwrap_or(false) { 0.2 } else { 1.0 };
                styled!(StyleVar::Alpha(image_alpha), $ui => {
                    Image::new($ctx.get_texture_id_for(&sprite), [Self::CELL, Self::CELL]).build($ui);
                });

                if let Some(stack) = $data.stacks.get(*item) {
                    let count = ImString::from(format!("{}", stack.current));
                    let [text_width, text_height] = $ui.calc_text_size(&count, false, 0.0);
                    $ui.set_cursor_pos([Self::CELL + $pos[0] - text_width, Self::CELL + $pos[1] - text_height]);
                    $ui.text(&count);
                }
            });

            if $ui.is_item_hovered_with_flags(ItemHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM) {
                within_tooltip!($ui => { item_tooltip!($self, *item, $ui, $ctx, $data); });
            }
        }

        if $ui.is_item_clicked(MouseButton::Left) {
            if $self.dragging_item().is_some() {
                true
            } else {
                if $item_box.is_some() {
                    $self.dragging_item.view = $item_box as &mut ItemBox;
                }
                false
            }
        } else {
            false
        }
    }};
}
macro_rules! items {
    ($self:expr, $inv:expr, $ui:expr, $ctx:expr, $data:expr) => {
        let per_row = (($ui.window_content_region_width() - Self::PAD) / (Self::PCELL)).ceil() as usize;
        for (i, mut item_box) in $inv.content.iter_mut().enumerate() {
            let pos = [Self::PAD + Self::PCELL * (i % per_row) as f32, Self::PAD + Self::PCELL * (i / per_row) as f32];
            if item_box!($self, &mut item_box, pos, $ui, $ctx, $data) {
                drag_and_drop!(&mut item_box, &mut $self.dragging_item, $data);
            }
        }
    };
}
impl<'a> UiBuilder<&mut UiData<'a>> for InventoryWindow {
    fn build<'ctx>(&mut self, ui: &mut Ui, ctx: &mut UiContext<'ctx>, data: &mut UiData<'a>) {
        let mut for_deletion: Set<Entity> = Set::new();
        for e in &self.show_inventories_for {
            let mut is_opened = true;
            within_window!(Window::new(&ImString::new(format!("Inventory##{}", e.id())))
            .resizable(false)
            .focus_on_appearing(true)
            .opened(&mut is_opened)
            .size([0.0, 0.0], Condition::Once), &ui => {
                if let Some(mut inventory) = data.inventories.get_mut(*e) {
                    ui.bullet_text(im_str!("Content:"));
                    within_window!(ChildWindow::new("inv").size([380.0, 260.0]).border(true), &ui => {
                        if inventory.content.is_empty() {
                            let txt = im_str!("Empty !");
                            let [max_w, max_h] = ui.content_region_max();
                            let [text_w, text_h] = ui.calc_text_size(txt, false, 0.0);
                            ui.set_cursor_pos([(max_w - text_w) * 0.5, (max_h - text_h) * 0.5]);
                            ui.text(txt);
                        } else {
                            items!(self, &mut inventory, ui, ctx, data);
                        }
                    });
                }
                if let Some(hotbar) = data.hotbars.get_mut(*e) {
                    ui.bullet_text(im_str!("Hotbar:"));
                    within_window!(ChildWindow::new("hotbar").size([380.0, 70.0]), &ui => {
                        let slots_size = hotbar.content.len();
                        let [max_w, _] = ui.content_region_max();
                        for (i, mut item_box) in hotbar.content.iter_mut().enumerate() {
                            let pos = [
                                (Self::PCELL * i as f32) + (max_w - (Self::PCELL) * slots_size as f32) * 0.5,
                                Self::PAD
                            ];

                            if item_box!(self, item_box, pos, ui, ctx, data) {
                                let item = self.dragging_item().unwrap();
                                if data.consumables.get(item).is_some() {
                                    drag_and_drop!(&mut item_box, &mut self.dragging_item, data);
                                }
                            }
                        }
                    });
                }
                if let Some(weaponry) = data.weaponries.get_mut(*e) {
                    ui.bullet_text(im_str!("Weapons:"));
                    within_window!(ChildWindow::new("weapon").size([380.0, 70.0]), &ui => {
                        let mut slots = [&mut weaponry.primary, &mut weaponry.secondary];
                        let slots_size = slots.len();
                        let [max_w, _] = ui.content_region_max();
                        for (i, mut item_box) in slots.iter_mut().enumerate() {
                            let pos = [
                                (Self::PCELL * i as f32) + (max_w - (Self::PCELL) * slots_size as f32) * 0.5,
                                Self::PAD
                            ];

                            if item_box!(self, item_box, pos, ui, ctx, data) {
                                let item = self.dragging_item().unwrap();
                                if data.wpn_props.get(item).is_some() && data.wpn_attacks.get(item).is_some() {
                                    drag_and_drop!(&mut item_box, &mut self.dragging_item, data);
                                }
                            }
                        }
                    });
                }
            });
            if !is_opened || (!data.inventories.contains(*e) && !data.weaponries.contains(*e)) {
                for_deletion.insert(*e);
            }
        }

        for e in for_deletion {
            self.show_inventories_for.remove(&e);
        }
    }
}
