use super::system::UiBuilder;
use crate::entity;
use imgui::*;

#[derive(Default, Debug)]
pub struct DebugWindow {
    pub selected_entity: Option<&'static str>,
}
impl UiBuilder for DebugWindow {
    fn build(&mut self, ui: &mut imgui::Ui) {
        use log::debug;
        debug!("selected: {:?}", self.selected_entity);
        Window::new(im_str!("Debug window"))
            .resizable(false)
            .size([300.0, 0.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text(im_str!("Spawn entity:"));
                ChildWindow::new("spawn_entity")
                    .size([0.0, 100.0])
                    .build(&ui, || {
                        for id in &entity::IDS {
                            if *id == "player" {
                                continue;
                            }

                            let label = ImString::new(*id);
                            if Selectable::new(&label)
                                .selected(self.selected_entity == Some(*id))
                                .build(&ui)
                            {
                                self.selected_entity = Some(*id);
                            }
                        }
                    });
            });
    }
}
