use ggez::{
    event::{Axis, Button, EventHandler},
    input::{
        gamepad::GamepadId,
        keyboard::{KeyCode, KeyMods},
        mouse::MouseButton,
    },
    Context, GameResult,
};

pub enum SceneCommand {
    Push(fn(&mut ggez::Context) -> Box<dyn Scene>),
    ReplaceAll(fn(&mut ggez::Context) -> Box<dyn Scene>),
}
pub trait Scene: EventHandler {
    fn next_command(&self) -> Option<SceneCommand>;
    fn draw_prev(&self) -> bool;
}

pub struct SceneManager {
    commands: Vec<SceneCommand>,
    stack: Vec<Box<dyn Scene>>,
}
impl SceneManager {
    pub fn new() -> Self { Self { commands: vec![], stack: vec![] } }

    pub fn send_command(&mut self, command: SceneCommand) { self.commands.push(command); }

    fn current(&mut self) -> Option<&mut dyn Scene> {
        match self.stack.last_mut() {
            Some(b) => Some(b.as_mut()),
            None => None,
        }
    }
}
impl EventHandler for SceneManager {
    fn update(&mut self, context: &mut Context) -> GameResult {
        for command in self.commands.drain(..) {
            let mut new = match command {
                SceneCommand::Push(func) => func(context),
                SceneCommand::ReplaceAll(func) => {
                    self.stack.clear();
                    func(context)
                },
            };
            let size = ggez::graphics::window(context).get_inner_size().unwrap();
            new.resize_event(context, size.width as f32, size.height as f32);
            self.stack.push(new);
        }

        if let Some(scene) = self.current() {
            scene.update(context)?;
            if let Some(command) = scene.next_command() {
                self.send_command(command);
            }
        }
        Ok(())
    }

    fn draw(&mut self, context: &mut Context) -> GameResult {
        for scene in self.stack.iter_mut().rev() {
            scene.draw(context)?;
            if !scene.draw_prev() {
                break;
            }
        }
        Ok(())
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if let Some(scene) = self.current() {
            scene.mouse_button_down_event(ctx, button, x, y);
        }
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if let Some(scene) = self.current() {
            scene.mouse_button_up_event(ctx, button, x, y);
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        if let Some(scene) = self.current() {
            scene.mouse_motion_event(ctx, x, y, dx, dy);
        }
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        if let Some(scene) = self.current() {
            scene.mouse_wheel_event(ctx, x, y);
        }
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymods: KeyMods, repeat: bool) {
        if let Some(scene) = self.current() {
            scene.key_down_event(ctx, keycode, keymods, repeat);
        }
    }

    fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymods: KeyMods) {
        if let Some(scene) = self.current() {
            scene.key_up_event(ctx, keycode, keymods);
        }
    }

    fn text_input_event(&mut self, ctx: &mut Context, character: char) {
        if let Some(scene) = self.current() {
            scene.text_input_event(ctx, character);
        }
    }

    fn gamepad_button_down_event(&mut self, ctx: &mut Context, btn: Button, id: GamepadId) {
        if let Some(scene) = self.current() {
            scene.gamepad_button_down_event(ctx, btn, id);
        }
    }

    fn gamepad_button_up_event(&mut self, ctx: &mut Context, btn: Button, id: GamepadId) {
        if let Some(scene) = self.current() {
            scene.gamepad_button_up_event(ctx, btn, id);
        }
    }

    fn gamepad_axis_event(&mut self, ctx: &mut Context, axis: Axis, value: f32, id: GamepadId) {
        if let Some(scene) = self.current() {
            scene.gamepad_axis_event(ctx, axis, value, id);
        }
    }

    fn focus_event(&mut self, ctx: &mut Context, gained: bool) {
        if let Some(scene) = self.current() {
            scene.focus_event(ctx, gained);
        }
    }

    fn quit_event(&mut self, ctx: &mut Context) -> bool {
        if let Some(scene) = self.current() {
            scene.quit_event(ctx)
        } else {
            false
        }
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        if let Some(scene) = self.current() {
            scene.resize_event(ctx, width, height);
        }
    }
}
