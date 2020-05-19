use super::system::UiBuilder;
use crate::entity;
use imgui::*;

pub struct DebugToolsUi;
impl UiBuilder for DebugToolsUi {
    fn build(&self, ui: &mut imgui::Ui) {
        Window::new(im_str!("Debug window"))
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(ui, || {
                let owned_ids: Vec<ImString> = entity::IDS
                    .iter()
                    .filter(|id| id != &&"player")
                    .map(|id| ImString::new(id.to_owned()))
                    .collect();

                let mut ids_view: Vec<&ImString> = vec![];
                for id in &owned_ids {
                    ids_view.push(id);
                }

                ui.text(im_str!("Spawn entity:"));
                let mut selected = 0;
                ui.list_box::<ImString>(im_str!(""), &mut selected, &ids_view, 4);
                ui.separator();
            });
    }
}
