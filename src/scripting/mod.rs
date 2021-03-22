use bevy::{log, prelude::*};

mod rune;

pub use self::rune::RuneContext;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<rune::RuneErrorEvent>()
            .add_resource(rune::RuneContext::new())
            .add_system(rune::run_rune_script_system.system())
            .add_system(rune::log_rune_errors_system.system());
    }
}
