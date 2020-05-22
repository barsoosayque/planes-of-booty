use anyhow::Result;
use codegen::*;
use def::*;
use std::{collections::BTreeSet as Set, fs};

mod def;

fn parse_entity_def(name: &str, data: &str) -> Result<EntityDef> {
    Ok(EntityDef { name: name.to_owned(), ..serde_yaml::from_str::<EntityDef>(data)? })
}

fn generate_generic_spawn_fn(defs: &Vec<EntityDef>) -> Function {
    let mut fn_gen = Function::new("spawn");
    fn_gen.arg("id", "&str").arg("world", "&specs::World").arg("ctx", "&mut ggez::Context");
    fn_gen.ret("specs::Entity");
    fn_gen.vis("pub");
    fn_gen.allow("dead_code");

    fn_gen.line("match id {");
    for def in defs {
        fn_gen.line(&format!("\"{}\" => spawn_{}(world, ctx),", def.name, def.name));
    }
    fn_gen.line("_ => panic!(\"Unknown id for spawning an entity: {}\", id),");
    fn_gen.line("}");
    fn_gen
}

fn generate_generic_view_fn(defs: &Vec<EntityDef>) -> Function {
    let mut fn_gen = Function::new("view");
    fn_gen.arg("id", "&str").arg("ctx", "&mut ggez::Context").arg("assets", "&mut crate::assets::AssetManager");
    fn_gen.ret("Option<(std::sync::Arc<crate::assets::ImageAsset>, crate::math::Size2f)>");
    fn_gen.vis("pub");
    fn_gen.allow("dead_code");

    fn_gen.line("match id {");
    for def in defs {
        match get_view_from(def, "Sprite", "asset") {
            Some(asset) => fn_gen.line(&format!("\"{}\" => Some(({}, {})),", def.name, asset.0, asset.1)),
            None => fn_gen.line(&format!("\"{}\" => None,", def.name)),
        };
    }
    fn_gen.line("_ => panic!(\"Unknown id for spawning an entity: {}\", id),");
    fn_gen.line("}");
    fn_gen
}

fn generate_spawn_fn(def: &EntityDef) -> Function {
    // recursivly collect inits and fins
    fn collect_init_and_fin(part: &PartValue, buffers: &mut (Set<String>, Set<String>)) {
        if let Some(init) = part.initialize() {
            buffers.0.insert(init);
        }
        if let Some(fin) = part.finalize() {
            buffers.1.insert(fin);
        }
        match part {
            PartValue::Seq(vec) => {
                for part in vec {
                    collect_init_and_fin(part, buffers);
                }
            },
            PartValue::Directional { north, east, west, south } => {
                collect_init_and_fin(north, buffers);
                collect_init_and_fin(east, buffers);
                collect_init_and_fin(west, buffers);
                collect_init_and_fin(south, buffers);
            },
            PartValue::Single { value } => collect_init_and_fin(value, buffers),
            PartValue::Collide { shape, .. } => collect_init_and_fin(shape, buffers),
            _ => (),
        }
    }
    // collect unique initialize and finalize lines from sorted parts
    let mut buffers: (Set<String>, Set<String>) = (Set::new(), Set::new());
    let mut sorted_parts: Vec<&PartValue> =
        def.components.values().flat_map(|contents| contents.parts.values()).collect();
    sorted_parts.sort_unstable_by(|a, b| {
        if a.is_dependent(b) {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    });
    for part in sorted_parts {
        collect_init_and_fin(part, &mut buffers);
    }

    let mut fn_gen = Function::new(&format!("spawn_{}", def.name));
    fn_gen.arg("world", "&specs::World").arg("ctx", "&mut ggez::Context");
    fn_gen.ret("specs::Entity");
    fn_gen.vis("pub");
    fn_gen.allow("dead_code");

    fn_gen.line("use specs::{WorldExt,world::Builder};");
    fn_gen.line("let mut assets = world.write_resource::<crate::assets::AssetManager>();");
    for line in buffers.0 {
        fn_gen.line(&line);
    }

    fn_gen.line("let entity = world.create_entity_unchecked()");
    for tag in &def.tags {
        fn_gen.line(format!(".with(tag::{})", tag));
    }
    for (name, contents) in &def.components {
        let component = if contents.default && contents.parts.len() == 0 {
            format!("component::{}::default()", name)
        } else {
            let mut body = format!("component::{}{{", name);
            body.push_str(
                &contents
                    .parts
                    .iter()
                    .map(|(part_name, part_value)| format!("{}:{}", part_name, part_value))
                    .collect::<Vec<String>>()
                    .join(","),
            );
            if contents.default {
                body.push_str(&format!(",..component::{}::default()", name));
            }
            body.push_str("}");
            body
        };

        fn_gen.line(format!(".with({})", component));
    }
    fn_gen.line(".build();");
    for line in buffers.1 {
        fn_gen.line(&line);
    }

    fn_gen.line("entity");
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
            },
            Err(err) => {
                println!("Error !");
                panic!("Error while parsing entity definition file {:?}: {}", path, err);
            },
        }
    }

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let mut scope = Scope::new();
    {
        let names: Vec<String> = defs.iter().map(|def| format!("\"{}\"", def.name)).collect();
        scope.raw(&format!(
            "#[allow(dead_code)] pub const IDS: [&'static str; {}] = [{}];",
            names.len(),
            names.join(",")
        ));
    }
    scope.push_fn(generate_generic_spawn_fn(&defs));
    scope.push_fn(generate_generic_view_fn(&defs));
    for def in &defs {
        scope.push_fn(generate_spawn_fn(&def));
    }
    fs::write(format!("{}/entity_gen.rs", out_dir), scope.to_string()).unwrap();

    println!("Total entities: {}", defs.len());
}
