use super::{stage, State, States};
use bevy::prelude::*;
use bevy_megaui::{
    megaui::{hash, widgets, Color, Style, Vector2},
    MegaUiContext, MegaUiPlugin,
};

pub struct MainMenuPlugin;

struct MainMenuUi {
    base_window_style: Style,
    defaut_style: Style,
}

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let mut window_style = Style::default();
        let color_none = Color::new(0.0, 0.0, 0.0, 0.0);
        window_style.window_background_focused = color_none;
        window_style.window_background_inactive = color_none;

        let mut default_style = Style::default();
        let color_text = Color::from_rgb(200, 160, 185);
        let color_text_focused = Color::from_rgb(235, 110, 115);
        default_style.inactive_text = color_text;
        default_style.inactive_title = color_text;
        default_style.focused_text = color_text_focused;
        default_style.focused_title = color_text_focused;

        default_style.button_background_focused = Color::from_rgb(60, 60, 60);
        default_style.button_background_focused_hovered = Color::from_rgb(90, 90, 90);
        default_style.button_background_focused_clicked = Color::from_rgb(40, 40, 40);

        let ui = MainMenuUi { base_window_style: window_style, defaut_style: default_style };

        app.add_plugin(MegaUiPlugin)
            .add_resource(ui)
            .on_state_enter(stage::APP_STATE, State::MainMenu, Self::setup.system())
            .on_state_exit(stage::APP_STATE, State::MainMenu, Self::destruct.system())
            .on_state_update(stage::APP_STATE, State::MainMenu, Self::ui_system.system());
    }
}

impl MainMenuPlugin {
    fn setup() {
        info!("Main Menu: initialized");
    }

    fn destruct() {
        info!("Main Menu: destructed");
    }

    fn ui_system(_: &mut World, resources: &mut Resources) {
        let mut context = resources.get_thread_local_mut::<MegaUiContext>().unwrap();
        let windows = resources.get::<Windows>().unwrap();
        let window = windows.get_primary().unwrap();
        let main_menu_ui = resources.get::<MainMenuUi>().unwrap();

        context.ui.set_style(main_menu_ui.base_window_style.clone());
        widgets::Window::new(hash!(), Vector2::default(), Vector2::new(window.width(), window.height()))
            .movable(false)
            .titlebar(false)
            .ui(&mut context.ui, |ui| {
                ui.set_style(main_menu_ui.defaut_style.clone());

                ui.label(None, &format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")));
                if ui.button(None, "play") {
                    let mut states = resources.get_mut::<States>().unwrap();
                    states.set_next(State::Game).unwrap();
                }
            });
    }
}
