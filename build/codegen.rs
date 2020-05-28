use crate::def::{EntityDef, PartValue};
use codegen::*;
use std::collections::BTreeSet as Set;

pub fn generate_full_group(defs: &Vec<EntityDef>, group_name: &str) -> String {
    let mut scope = Scope::new();
    scope.raw(&generate_names_array(defs));
    scope.push_fn(generate_generic_spawn_fn(&defs));
    scope.push_fn(generate_generic_view_fn(&defs));
    for def in defs {
        scope.push_fn(generate_spawn_fn(&def, group_name));
    }
    scope.to_string()
}

pub fn generate_names_array(defs: &Vec<EntityDef>) -> String {
    let names: Vec<String> = defs.iter().map(|def| format!("\"{}\"", def.name)).collect();
    format!("#[allow(dead_code)] pub const IDS: [&'static str; {}] = [{}];", names.len(), names.join(","))
}

pub fn generate_generic_spawn_fn(defs: &Vec<EntityDef>) -> Function {
    let mut fn_gen = Function::new("spawn");
    fn_gen
        .arg("id", "&str")
        .arg("world", "&specs::World")
        .arg("ctx", "&mut ggez::Context")
        .arg("assets", "&mut crate::assets::AssetManager");
    fn_gen.ret("specs::Entity");
    fn_gen.vis("pub");
    fn_gen.allow("dead_code");

    fn_gen.line("match id {");
    for def in defs {
        fn_gen.line(&format!("\"{}\" => spawn_{}(world, ctx, assets),", def.name, def.name));
    }
    fn_gen.line("_ => panic!(\"Unknown id for spawning an entity: {}\", id),");
    fn_gen.line("}");
    fn_gen
}

fn get_view_from<'a>(
    def: &'a EntityDef,
    component_name: &str,
    asset_part: &str,
) -> Option<(&'a PartValue, &'a PartValue)> {
    def.components.get(component_name).map(|comp| {
        (
            comp.parts
                .get(asset_part)
                .map(|part| match part {
                    PartValue::Single { value } => value.as_ref(),
                    PartValue::Directional { north, .. } => north.as_ref(),
                    _ => panic!("{} should be either single or directional", component_name),
                })
                .expect(&format!("{} field is missing for component {} in {}", asset_part, component_name, def.name)),
            comp.parts
                .get("size")
                .expect(&format!("width field is missing component for {} in {}", component_name, def.name)),
        )
    })
}

pub fn generate_generic_view_fn(defs: &Vec<EntityDef>) -> Function {
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

// utility: recursivly collect inits and fins from part value
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

pub fn generate_spawn_fn(def: &EntityDef, reflection_prefix: &str) -> Function {
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
    fn_gen
        .arg("world", "&specs::World")
        .arg("ctx", "&mut ggez::Context")
        .arg("assets", "&mut crate::assets::AssetManager");
    fn_gen.ret("specs::Entity");
    fn_gen.vis("pub");
    fn_gen.allow("dead_code");

    fn_gen.line("use specs::{WorldExt,world::Builder};");
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
    fn_gen.line(&format!(".with(component::Reflection{{id:\"{}_{}\"}})", reflection_prefix, def.name));
    fn_gen.line(".build();");
    for line in buffers.1 {
        fn_gen.line(&line);
    }

    fn_gen.line("entity");
    fn_gen
}
