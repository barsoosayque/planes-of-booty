use bevy::prelude::*;
use smallvec::SmallVec;

mod main_menu;

pub enum State {
    MainMenu,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut AppBuilder) {
        let mut state_commands = StateCommands::default();
        state_commands.push(State::MainMenu);

        app.add_plugin(main_menu::MainMenuPlugin)
            .add_system_to_stage(stage::PRE_UPDATE, Self::watch_commands_system.system())
            .add_resource(state_commands);
    }
}

impl StatePlugin {
    fn watch_commands_system(
        mut commands: Commands,
        mut stack: Local<Vec<Entity>>,
        mut state_commands: ResMut<StateCommands>,
    ) {
        for command in state_commands.commands.drain(..) {
            match command {
                StateCommand::Push(state) => {
                    let comp = match state {
                        State::MainMenu => main_menu::MainMenuComponent,
                    };
                    commands.spawn((comp,));
                    stack.push(commands.current_entity().unwrap());
                },
                StateCommand::Pop => {
                    if let Some(entity) = stack.pop() {
                        commands.despawn(entity);
                    }
                },
                StateCommand::Clear => {
                    for entity in stack.drain(..) {
                        commands.despawn(entity);
                    }
                },
            };
        }
    }
}
#[derive(Default)]
pub struct StateCommands {
    commands: SmallVec<[StateCommand; 4]>,
}
enum StateCommand {
    Push(State),
    Pop,
    Clear,
}

impl StateCommands {
    pub fn push(&mut self, state: State) { self.commands.push(StateCommand::Push(state)); }

    pub fn pop(&mut self) { self.commands.push(StateCommand::Pop); }

    pub fn clear(&mut self) { self.commands.push(StateCommand::Clear); }
}
