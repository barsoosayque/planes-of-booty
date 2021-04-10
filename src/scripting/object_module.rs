use runestick::Module;

use crate::object::CameraObjectDef;

pub fn module() -> Module {
    let mut module = Module::default();
    module.ty::<CameraObjectDef>().unwrap();

    module
}