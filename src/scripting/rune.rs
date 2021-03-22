use bevy::prelude::*;
use lazy_static::lazy_static;
use rune::{Diagnostics, EmitDiagnostics, Options, Sources};
use runestick::{Context, Hash, Module, Source, Vm};
use std::{ops::DerefMut, sync::Arc};

lazy_static! {
    static ref MAIN_HASH: Hash = Hash::type_hash(&["main"]);
}

pub struct RuneErrorEvent {
    msg: String,
}

pub struct RuneContext {
    ctx: Context,
    queue: Vec<(&'static str, &'static str)>,
}

impl RuneContext {
    pub fn new() -> Self {
        let mut ctx = Context::default();
        let mut module = Module::default();
        module.function(&["debug"], |s: String| debug!("{}", &s)).unwrap();
        module.function(&["info"], |s: String| info!("{}", &s)).unwrap();
        ctx.install(&module).unwrap();

        Self { ctx, queue: vec![] }
    }

    pub fn run_script(&mut self, name: &'static str, script: &'static str) {
        self.queue.push((name, script));
    }
}

macro_rules! send_diagnostics {
    ($diagnostics:expr, $sources:expr => $events:expr) => {
        if !$diagnostics.is_empty() {
            let mut out = termcolor::NoColor::new(Vec::<u8>::new());
            $diagnostics.emit_diagnostics(&mut out, &$sources).unwrap();
            let str = unsafe { String::from_utf8_unchecked(out.into_inner()) };
            $events.send(RuneErrorEvent { msg: str });
        }
    };
}

pub fn run_rune_script_system(
    mut ctx: ResMut<RuneContext>,
    mut events: ResMut<Events<RuneErrorEvent>>,
) {
    let RuneContext { ctx, queue } = ctx.deref_mut();

    for (name, script) in queue.drain(..) {
        let mut diagnostics = Diagnostics::new();
        let mut sources = Sources::new();
        sources.insert(Source::new(name, script));

        let result = rune::load_sources(&ctx, &Options::default(), &mut sources, &mut diagnostics);
        let unit = match result {
            Ok(unit) => unit,
            Err(_) => {
                send_diagnostics!(diagnostics, sources => events);
                continue;
            },
        };

        let vm = Vm::new(Arc::new(ctx.runtime()), Arc::new(unit));
        if let Err(_) = vm.call(*MAIN_HASH, ()) {
            send_diagnostics!(diagnostics, sources => events);
        }
    }
}

#[derive(Default)]
pub struct LogRuneErrorsSystemState {
    reader: EventReader<RuneErrorEvent>,
}
pub fn log_rune_errors_system(
    mut state: Local<LogRuneErrorsSystemState>,
    mut events: ResMut<Events<RuneErrorEvent>>,
) {
    for event in state.reader.iter(&mut events) {
        error!("Rune script error:\n{}", &event.msg);
    }
}
