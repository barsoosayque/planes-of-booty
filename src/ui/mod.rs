pub mod arena_settings;
pub mod debug;
pub mod game_over;
pub mod hud;
pub mod inventory;
pub mod main_menu;
pub mod menu;
pub mod pause;
pub mod system;

pub use arena_settings::ArenaSettingsWindow;
pub use debug::DebugWindow;
pub use game_over::GameOverWindow;
pub use hud::Hud;
pub use inventory::InventoryWindow;
pub use main_menu::MainMenu;
pub use menu::Menu;
pub use pause::PauseWindow;
pub use system::{ImGuiSystem, UiBuilder, UiContext};

#[macro_export]
macro_rules! within_window {
    ($builder:expr, $ui:expr => $block:block) => {
        if let Some(token) = $builder.begin($ui) {
            $block;
            token.end($ui);
        }
    };
}

#[macro_export]
macro_rules! within_group {
    ($ui:expr => $block:block) => {
        let token = $ui.begin_group();
        $block;
        token.end($ui);
    };
}

#[macro_export]
macro_rules! within_tooltip {
    ($ui:expr => $block:block) => {
        let token = $ui.begin_tooltip();
        $block;
        token.end($ui);
    };
}

#[macro_export]
macro_rules! styled {
    ($style:expr, $ui:expr => $block:block) => {
        let token = $ui.push_style_var($style);
        $block;
        token.pop($ui);
    };
}

#[macro_export]
macro_rules! centered_text {
    ($ui:expr; $text:expr; width,height) => {
        let [w, h] = $ui.calc_text_size(&ImString::new($text), true, 0.0);
        let [ww, wh] = $ui.window_size();
        $ui.set_cursor_pos([(ww - w) * 0.5, (wh - h) * 0.5]);
        $ui.text($text);
    };
    ($ui:expr; $text:expr; width) => {
        let pos = $ui.cursor_pos();
        let [w, _] = $ui.calc_text_size(&ImString::new($text), true, 0.0);
        let [ww, _] = $ui.window_size();
        $ui.set_cursor_pos([(ww - w) * 0.5, pos[1]]);
        $ui.text($text);
    };
}
