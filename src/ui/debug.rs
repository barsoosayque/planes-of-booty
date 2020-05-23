use super::system::{UiBuilder, TextureProvider};
use crate::{ecs::resource::*, entity, item};
use imgui::*;
use specs::Entity;

#[derive(Debug)]
pub struct DebugWindow {
    pub selected_entity: Option<&'static str>,
    pub selected_item: Option<&'static str>,
    pub item_spawn_count: i32,
}
impl Default for DebugWindow {
    fn default() -> Self { Self { selected_item: None, selected_entity: None, item_spawn_count: 1 } }
}
impl<'a> UiBuilder<'a> for DebugWindow {
    type Data = (&'a Entity, &'a mut SpawnQueue);

    fn build(&mut self, ui: &mut imgui::Ui, tex: &mut TextureProvider<'a>, (player, spawn_queue): Self::Data) {
        Window::new(im_str!("Debug window"))
            .position_pivot([1.0, 0.0])
            .resizable(false)
            .focus_on_appearing(false)
            .size([300.0, 0.0], Condition::Once)
            .build(ui, || {
                ui.text(im_str!("Spawn entity:"));
                ChildWindow::new("spawn_entity").size([0.0, 100.0]).build(&ui, || {
                    for id in &entity::IDS {
                        if *id == "player" {
                            continue;
                        }

                        let label = ImString::new(*id);
                        if Selectable::new(&label).selected(self.selected_entity == Some(*id)).build(&ui) {
                            self.selected_entity = Some(*id);
                        }
                    }
                });
                ui.separator();

                ui.text(im_str!("Add item to inventory:"));
                ui.columns(2, im_str!("add_item_col"), false);
                ui.set_current_column_width(150.0);
                ChildWindow::new("add_item").size([0.0, 100.0]).build(&ui, || {
                    for id in &item::IDS {
                        let label = ImString::new(*id);
                        if Selectable::new(&label).selected(self.selected_item == Some(*id)).build(&ui) {
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
                    if let Some(id) = self.selected_item {
                        spawn_queue.0.push_back(SpawnItem::Item(id.into(), self.item_spawn_count as u32, *player));
                    }
                }
            });
    }
}
