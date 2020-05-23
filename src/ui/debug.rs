use super::system::UiBuilder;
use crate::{entity,item};
use imgui::*;

#[derive(Default, Debug)]
pub struct DebugWindow {
    pub selected_entity: Option<&'static str>,
    pub selected_item: Option<&'static str>,
    pub item_spawn_count: i32
}
impl<'a> UiBuilder<'a> for DebugWindow {
    type Data = ();

    fn build(&mut self, ui: &mut imgui::Ui, _: Self::Data) {
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

                ui.text(im_str!("Add item to invetory:"));
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
                    self.item_spawn_count = self.item_spawn_count.max(0);
                }
                if ui.button(im_str!("Add"), [150.0, 20.0]) {
                    // TODO: spawn items
                }
            });
    }
}
