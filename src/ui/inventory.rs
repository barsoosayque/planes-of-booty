use super::system::UiBuilder;
use crate::ecs::component::Inventory;
use imgui::*;

#[derive(Default, Debug)]
pub struct InventoryWindow {
    pub inventory: Option<&'static str>,
}
impl UiBuilder for InventoryWindow {
    fn build(&mut self, ui: &mut imgui::Ui) {
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
            });
    }
}
