use super::{stage, State::MainMenu};
use bevy::{prelude::*, window::Window};
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

        let ui = MainMenuUi { base_window_style: window_style, defaut_style: Style::default() };

        app.add_plugin(MegaUiPlugin)
            .add_resource(ui)
            .on_state_enter(stage::APP_STATE, MainMenu, Self::setup.system())
            .add_system(Self::ui_system.system());
    }
}

impl MainMenuPlugin {
    fn setup() {
        debug!("hi from main menu");
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
                ui.button(None, "play");
            });
    }
}
