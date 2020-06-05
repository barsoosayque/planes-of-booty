use super::system::{UiBuilder, UiContext};
use crate::{
    arena,
    ecs::{component::*, resource::*, tag},
};
use imgui::*;
use specs::Join;
use std::{fs::File, io::prelude::*};

#[derive(Default, Debug)]
pub struct ArenaSettingsWindow {
    pub selected_arena: Option<arena::ID>,
}
impl ArenaSettingsWindow {
    fn write_level_to_file<'a>(&mut self, data: &mut UiData<'a>) -> std::io::Result<()> {
        let mut file = File::create("level.yaml")?;
        file.write_all(
            format!("width: {:.1}\nheight: {:.1}\n", data.arena.size.width, data.arena.size.height).as_ref(),
        )?;
        let stream = (&data.reflections, &data.transforms, !&data.player_tag).join().collect::<Vec<_>>();
        if !stream.is_empty() {
            file.write_all(b"entities:\n")?;
            for (Reflection { id }, Transform { pos, .. }, _) in stream {
                let (kind, id) = id.split_at(2);
                match kind.chars().next() {
                    Some('e') => file.write_all(
                        format!("    - id: \"{}\"\n      pos: {{ x: {:.1}, y: {:.1} }}\n", id, pos.x, pos.y).as_ref(),
                    )?,
                    _ => (),
                }
            }
        }
        Ok(())
    }
}
impl<'a> UiBuilder<(&mut UiData<'a>, &mut bool)> for ArenaSettingsWindow {
    fn build<'ctx>(&mut self, ui: &mut Ui, _: &mut UiContext<'ctx>, (data, is_opened): (&mut UiData<'a>, &mut bool)) {
        Window::new(im_str!("Arena settings"))
            .resizable(false)
            .focus_on_appearing(false)
            .opened(is_opened)
            .size([0.0, 0.0], Condition::Once)
            .build(ui, || {
                if ui.button(im_str!("Kill all"), [300.0, 20.0]) {
                    for (e, _, _) in (&data.entities, &data.transforms, !&data.player_tag).join() {
                        data.to_destruct.insert(e, tag::PendingDestruction).unwrap();
                    }
                }
                ui.separator();
                ui.text(im_str!("Arena settings"));
                ui.drag_float(im_str!("Width"), &mut data.arena.size.width).min(0.0).build();
                ui.drag_float(im_str!("Height"), &mut data.arena.size.height).min(0.0).build();
                ui.separator();
                ui.text(im_str!("Set arena:"));
                ChildWindow::new("spawn_entity").size([0.0, 100.0]).border(true).build(&ui, || {
                    for id in &arena::IDS {
                        let label = ImString::new(format!("{:?}", id));
                        if Selectable::new(&label).selected(self.selected_arena == Some(*id)).build(&ui) {
                            self.selected_arena = Some(*id);
                        }
                    }
                });
                if ui.button(im_str!("Load"), [300.0, 20.0]) {
                    if let Some(id) = self.selected_arena {
                        data.arena.change_to = Some(id);
                    }
                }
                ui.separator();
                if ui.button(im_str!("Save current as level.yaml"), [300.0, 20.0]) {
                    self.write_level_to_file(data).unwrap();
                }
            });
    }
}
