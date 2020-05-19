use anyhow::Result;
use codegen::*;
use def::*;
use std::fs;

mod def;

fn parse_entity_def(name: &str, data: &str) -> Result<EntityDef> {
    Ok(EntityDef {
        name: name.to_owned(),
        ..serde_yaml::from_str::<EntityDef>(data)?
    })
}

fn generate_spawn_fn(def: &EntityDef) -> Function {
    let mut fn_gen = Function::new(&format!("spawn_{}", def.name));
    fn_gen
        .arg("world", "&mut specs::World")
        .arg("ctx", "&mut ggez::Context")
        .arg("assets", "&mut crate::assets::AssetManager");
    fn_gen.ret("specs::Entity");
    fn_gen.vis("pub");

    fn_gen.line("use specs::{WorldExt,world::Builder};");
    fn_gen.line("world.create_entity()");
    for tag in &def.tags {
        fn_gen.line(format!(".with(tag::{})", tag));
    }
    for (name, contents) in &def.components {
        // use struct definition syntax for struct initialization
        let component = if contents.default && contents.parts.len() == 0 {
            format!("component::{}::default()", name)
        } else {
            let mut body = format!("component::{}{{", name);
            body.push_str(
                &contents
                    .parts
                    .iter()
                    .map(|(part_name, part_value)| match part_value {
                        PartValue::Str(value) => format!("{}: {}", part_name, value),
                        PartValue::Num(value) => format!("{}: {}f32", part_name, value),
                        PartValue::Bool(value) => format!("{}: {}", part_name, value),
                        PartValue::Image(path) => format!(
                            "{}: assets.get::<crate::assets::ImageAsset>(\"{}\", ctx).unwrap()",
                            part_name, path
                        ),
                    })
                    .collect::<Vec<String>>()
                    .join(","),
            );
            if contents.default {
                body.push_str(&format!(", ..component::{}::default()", name));
            }
            body.push_str("}");
            body
        };

        fn_gen.line(format!(".with({})", component));
    }
    fn_gen.line(".build()");

    fn_gen
}

fn main() {
    let mut defs: Vec<EntityDef> = vec![];

    // Parse entities definition and generate apropriate functions
    for entry in fs::read_dir("resources/entities").unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_file() {
            println!("Skipping {:?}, not a file.. ", entry.path());
            continue;
        }
        let path = entry.path();
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap();
        let content = fs::read_to_string(&path).unwrap();

        println!("cargo:rerun-if-changed={:?}", path);
        print!("Parsing file {:?} as entity.. ", path);
        match parse_entity_def(&name, &content) {
            Ok(def) => {
                println!("Success !");
                defs.push(def);
            }
            Err(err) => {
                println!("Error !");
                println!(
                    "cargo:warning=Error while parsing entity definition file {:?}: {}",
                    path, err
                );
            }
        }
    }

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let mut scope = Scope::new();
    for def in &defs {
        scope.push_fn(generate_spawn_fn(&def));
    }
    fs::write(format!("{}/entity_gen.rs", out_dir), scope.to_string()).unwrap();

    println!("Total entities: {}", defs.len());
}
