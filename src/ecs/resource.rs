use ggez::input;
use std::collections::HashSet;

#[derive(Default, Debug)]
pub struct DeltaTime(pub std::time::Duration);

#[derive(Default, Debug)]
pub struct Inputs {
    pub pressed_keys: HashSet<input::keyboard::KeyCode>,
}
