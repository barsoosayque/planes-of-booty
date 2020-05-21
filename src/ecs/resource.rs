use ggez::input;
use std::collections::HashSet;
use crate::ui::*;

#[derive(Default, Debug)]
pub struct DeltaTime(pub std::time::Duration);

#[derive(Default, Debug)]
pub struct Inputs {
    pub pressed_keys: HashSet<input::keyboard::KeyCode>,
}

#[derive(Default, Debug)]
pub struct UiHub {
    pub debug_window: DebugWindow
}
impl UiBuilder for UiHub {
    fn build(&mut self, ui: &mut imgui::Ui) {
        self.debug_window.build(ui);
    }
}
