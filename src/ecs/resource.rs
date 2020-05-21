use crate::math::Point2f;
use crate::ui::*;
use ggez::input;
use std::collections::HashSet;
use std::collections::VecDeque as Queue;

#[derive(Default, Debug)]
pub struct DeltaTime(pub std::time::Duration);

#[derive(Default, Debug)]
pub struct Inputs {
    pub pressed_keys: HashSet<input::keyboard::KeyCode>,
    pub mouse_clicked: HashSet<input::mouse::MouseButton>,
    pub mouse_pressed: HashSet<input::mouse::MouseButton>,
    pub mouse_pos: Point2f,
}

#[derive(Default, Debug)]
pub struct UiHub {
    pub menu: Menu,
    pub debug_window: DebugWindow,
}
impl UiBuilder for UiHub {
    fn build(&mut self, ui: &mut imgui::Ui) {
        self.menu.build(ui);

        if self.menu.is_show_spawn_window {
            self.debug_window.build(ui);
        }
    }
}

#[derive(Default, Debug)]
pub struct SpawnQueue(pub Queue<SpawnItem>);
#[derive(Default, Debug)]
pub struct SpawnItem {
    pub id: String,
    pub pos: Point2f,
}
