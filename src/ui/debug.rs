use super::system::{UiBuilder, UiContext};
use crate::{ecs::resource::*, entity, item};
use imgui::*;
use specs::Join;

#[derive(Debug)]
pub struct DebugWindow {
    pub selected_entity: Option<entity::ID>,
    pub selected_item: Option<item::ID>,
    pub item_spawn_count: i32,
}
impl Default for DebugWindow {
    fn default() -> Self { Self { selected_item: None, selected_entity: None, item_spawn_count: 1 } }
}
impl<'a> UiBuilder<&mut UiData<'a>> for DebugWindow {
    fn build<'ctx>(&mut self, ui: &mut Ui, ctx: &mut UiContext<'ctx>, data: &mut UiData<'a>) {
        Window::new(im_str!("Debug window"))
            .resizable(false)
            .focus_on_appearing(false)
            .size([300.0, 0.0], Condition::Once)
            .build(ui, || {
                ui.text(im_str!("Spawn entity:"));
                ChildWindow::new("spawn_entity").size([0.0, 100.0]).border(true).build(&ui, || {
                    for id in &entity::IDS {
                        if *id == entity::ID::Player  {
                            continue;
                        }

                        let label = ImString::new(format!("{:?}", id));
                        if Selectable::new(&label).selected(self.selected_entity == Some(*id)).build(&ui) {
                            self.selected_entity = Some(*id);
                        }
                    }
                });
                ui.separator();

                ui.text(im_str!("Add item to inventory:"));
                ui.columns(2, im_str!("add_item_col"), false);
                ui.set_current_column_width(150.0);
                ChildWindow::new("add_item").size([0.0, 100.0]).border(true).build(&ui, || {
                    ui.columns(2, im_str!("add_item_col_inner"), false);
                    ui.set_current_column_width(30.0);
                    for id in &item::IDS {
                        let (asset, _) = item::view(*id, ctx.as_mut(), &mut data.assets).unwrap();
                        let tex_id = ctx.get_texture_id_for(&asset);
                        Image::new(tex_id, [30.0, 30.0]).build(ui);
                    }
                    ui.next_column();
                    ui.set_current_column_width(120.0);
                    for id in &item::IDS {
                        let label = ImString::new(format!("{:?}", id));
                        if Selectable::new(&label).selected(self.selected_item == Some(*id)).size([0.0, 30.0]).build(&ui) {
                            self.selected_item = Some(*id);
                        }
                    }
                });
                ui.next_column();
                ui.set_current_column_width(150.0);
                if ui.input_int(im_str!("Count"), &mut self.item_spawn_count).build() {
                    self.item_spawn_count = self.item_spawn_count.max(1);
                }
                if ui.button(im_str!("Add"), [150.0, 20.0]) {
                    use std::convert::TryInto;
                    if let (Some(id), Some((player, _))) = (self.selected_item, (&data.entities, &data.player_tag).join().next()) {
                        log::trace!("Add item {:?} x{} to player", id, self.item_spawn_count);
                        data.spawn_queue.0.push_back(SpawnItem::Item(id, self.item_spawn_count.try_into().unwrap(), player));
                    }
                }
            });
    }
}
