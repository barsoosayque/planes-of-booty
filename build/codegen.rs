use crate::def::{ArenaDef, ComponentDef, EntityDef, PartValue, SpawnGroupDef};
use codegen::*;
use heck::{CamelCase, ShoutySnakeCase, SnakeCase};
use itertools::Itertools;
use std::collections::{BTreeMap as Map, BTreeSet as Set};

// TODO: split this in separate functions in separate file
pub fn generate_arenas(arenas: &[ArenaDef]) -> Scope {
    let mut scope = Scope::new();
    {
        let mut fn_gen = Function::new("set");
        fn_gen.arg("id", "ID");
        fn_gen.arg("arena", "&mut resource::Arena");
        fn_gen.arg("spawn_queue", "&mut resource::SpawnQueue");
        fn_gen.vis("pub");
        fn_gen.line("match id {");
        for arena in arenas {
            fn_gen.line(&format!("ID::{} => set_{}(arena, spawn_queue),", arena.name.to_camel_case(), arena.name));
        }
        fn_gen.line("}");
        scope.push_fn(fn_gen);
    }
    for arena in arenas {
        let mut fn_gen = Function::new(&format!("set_{}", arena.name));
        fn_gen.arg("arena", "&mut resource::Arena");
        fn_gen.arg("spawn_queue", "&mut resource::SpawnQueue");
        fn_gen.vis("pub");
        fn_gen.line(format!("arena.size = crate::math::Size2f::new({}f32, {}f32);", arena.width, arena.height));
        fn_gen.line("use rand::{thread_rng, seq::SliceRandom, Rng};");
        for entity in &arena.entities {
            fn_gen.line(format!(
                "spawn_queue.0.push_back(resource::SpawnItem::Entity(entity::ID::{}, crate::math::Point2f::new({}, {}), vec![]));",
                entity.id.to_camel_case(), entity.pos.x, entity.pos.y
            ));
        }
        for point in &arena.spawn_points {
            let halfr = point.radius * 0.5;
            fn_gen.line("{");
            fn_gen.line("let mut rng = thread_rng();");
            fn_gen.line("let generated = SPAWN_GROUPS.choose(&mut rng).unwrap().spawn(arena.difficulty);");
            fn_gen.line("for id in generated {");
            fn_gen.line(format!(
                "spawn_queue.0.push_back(resource::SpawnItem::Entity(id,\
                crate::math::Point2f::new({}f32 + rng.gen_range({}f32, {}f32), {}f32 + rng.gen_range({}f32, {}f32)), vec![]));",
                point.pos.x, -halfr, halfr, point.pos.y, -halfr, halfr
            ));
            fn_gen.line("}\n}");
        }
        scope.push_fn(fn_gen);
    }
    let names = arenas.into_iter().map(|def| def.name.to_camel_case());
    scope.raw(&format!(
        "#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)] pub enum ID{{{}}}",
        names.clone().join(",")
    ));
    scope.raw(&format!("pub const IDS: [ID; {}] = [{}];", names.len(), names.map(|n| format!("ID::{}", n)).join(",")));
    scope
}

// TODO: split this as well
pub fn generate_spawn_groups(spawn_groups: &[SpawnGroupDef]) -> Scope {
    let mut scope = Scope::new();
    let names = spawn_groups.into_iter().map(|def| def.name.to_camel_case());
    scope.raw(&format!(
        "#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)] pub enum SpawnGroup{{{}}}",
        names.clone().join(",")
    ));
    for grp in spawn_groups {
        scope.raw(&format!(
            "const {}_CHOICES: [entity::ID; {}] = [{}];",
            grp.name.to_shouty_snake_case(),
            grp.weighted.len(),
            grp.weighted.iter().map(|w| format!("entity::ID::{}", w.id.to_camel_case())).join(",")
        ));
        scope.raw(&format!(
            "const {}_WEIGHTS: [u8; {}] = [{}];",
            grp.name.to_shouty_snake_case(),
            grp.weighted.len(),
            grp.weighted.iter().map(|w| format!("{}", w.weight)).join(",")
        ));
    }
    let impl_gen = scope.new_impl("SpawnGroup");
    let fn_gen = impl_gen.new_fn("spawn");
    fn_gen.vis("pub");
    fn_gen.arg_self();
    fn_gen.arg("difficulty", "f32");
    fn_gen.ret("Vec<entity::ID>");
    fn_gen.line("use rand::distributions::weighted::alias_method::WeightedIndex;");
    fn_gen.line("use rand::prelude::*;");
    fn_gen.line("let mut rng = thread_rng();");
    fn_gen.line("match self {");
    for grp in spawn_groups {
        let shouty = grp.name.to_shouty_snake_case();
        fn_gen.line(format!(
            "SpawnGroup::{} => {{\
                let dist = WeightedIndex::new({}_WEIGHTS.to_vec()).unwrap();\
                let size = {}u32 + (difficulty * {}f32).floor() as u32;\
                let mut v = Vec::with_capacity(size as usize);\
                for _ in 0..=size {{ v.push({}_CHOICES[dist.sample(&mut rng)]) }}\
                v }}",
            grp.name.to_camel_case(),
            shouty,
            grp.start,
            grp.grow,
            shouty
        ));
    }
    fn_gen.line("}");

    scope.raw(&format!(
        "pub const SPAWN_GROUPS: [SpawnGroup; {}] = [{}];",
        names.len(),
        names.map(|n| format!("SpawnGroup::{}", n)).join(",")
    ));
    scope
}

pub fn generate_full_group(defs: &[EntityDef], group_name: &str) -> Scope {
    let mut scope = Scope::new();
    scope.raw(&generate_names_enum(defs));
    scope.push_fn(generate_generic_spawn_fn(&defs));
    scope.push_fn(generate_generic_view_fn(&defs));
    for def in defs {
        scope.push_fn(generate_spawn_fn(&def, group_name));
    }
    scope
}

pub fn generate_spawn_only(defs: &[EntityDef], group_name: &str) -> Scope {
    let mut scope = Scope::new();
    scope.raw(&generate_names_enum(defs));
    scope.push_fn(generate_generic_spawn_fn(&defs));
    for def in defs {
        scope.push_fn(generate_spawn_fn(&def, group_name));
    }
    scope
}

pub fn generate_names_enum(defs: &[EntityDef]) -> String {
    let names = defs.into_iter().map(|def| def.name.to_camel_case());
    let r#enum = format!(
        "#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)] pub enum ID{{{}}}",
        names.clone().join(",")
    );
    let array = format!("pub const IDS: [ID; {}] = [{}];", names.len(), names.map(|n| format!("ID::{}", n)).join(","));
    format!("{}\n{}", r#enum, array)
}

pub fn generate_array_by_filter<F: FnMut(&&EntityDef) -> bool>(
    defs: &[EntityDef],
    array_name: &str,
    filter: F,
) -> String {
    let defs = defs.into_iter().filter(filter).collect_vec();
    format!(
        "pub const {}: [ID; {}] = [{}];",
        array_name,
        defs.len(),
        defs.iter().map(|d| format!("ID::{}", d.name.to_camel_case())).join(",")
    )
}

pub fn generate_generic_spawn_fn(defs: &[EntityDef]) -> Function {
    let mut fn_gen = Function::new("spawn");
    fn_gen
        .arg("id", "ID")
        .arg("world", "&specs::World")
        .arg("ctx", "&mut ggez::Context")
        .arg("assets", "&mut crate::assets::AssetManager");
    fn_gen.ret("specs::Entity");
    fn_gen.vis("pub");

    fn_gen.line("match id {");
    for def in defs {
        fn_gen.line(&format!("ID::{} => spawn_{}(world, ctx, assets),", def.name.to_camel_case(), def.name));
    }
    fn_gen.line("}");
    fn_gen
}

fn get_view_from<'a>(
    id: &str,
    from: &'a Map<String, ComponentDef>,
    component_name: &str,
    asset_part: &str,
) -> Option<(&'a PartValue, &'a PartValue)> {
    from.get(component_name).map(|comp| {
        (
            comp.parts
                .get(asset_part)
                .map(|part| match part {
                    PartValue::Single { value } => value.as_ref(),
                    PartValue::Directional { north, .. } => north.as_ref(),
                    _ => panic!("{} should be either single or directional", component_name),
                })
                .expect(&format!("{} field is missing for component {} ({})", asset_part, component_name, id)),
            comp.parts.get("size").expect(&format!("width field is missing component for {} ({})", component_name, id)),
        )
    })
}

pub fn generate_generic_view_fn(defs: &[EntityDef]) -> Function {
    let mut fn_gen = Function::new("view");
    fn_gen.arg("id", "ID").arg("ctx", "&mut ggez::Context").arg("assets", "&mut crate::assets::AssetManager");
    fn_gen.ret("Option<(std::sync::Arc<crate::assets::ImageAsset>, crate::math::Size2f)>");
    fn_gen.vis("pub");

    fn_gen.line("match id {");
    for def in defs {
        let name = def.name.to_camel_case();
        match get_view_from(&def.name, &def.components, "Sprite", "asset") {
            Some(asset) => fn_gen.line(&format!("ID::{} => Some(({}, {})),", name, asset.0, asset.1)),
            None => fn_gen.line(&format!("ID::{} => None,", name)),
        };
    }
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
        PartValue::Collide { shape, hitbox, .. } => {
            collect_init_and_fin(shape, buffers);
            if let Some(hitbox) = hitbox {
                collect_init_and_fin(hitbox, buffers);
            }
        },
        _ => (),
    }
}
fn stringify_component(name: &str, contents: &ComponentDef) -> String {
    if contents.default && contents.parts.len() == 0 {
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

    fn_gen.line("use specs::{WorldExt,world::Builder};");
    for line in buffers.0 {
        fn_gen.line(&line);
    }

    for (name, contents) in &def.shared_components {
        let contents = stringify_component(&name, &contents);
        let shared_field = format!("SHARED_{}", name.to_shouty_snake_case());
        fn_gen.line(format!("static mut {}: Option<std::sync::Arc<component::{}>> = None;", shared_field, name));
        let lazy_init = format!(
            "let shared_{} = unsafe{{match&{}{{
                Some(arc) => arc.clone(),
                None => {{{}.replace(std::sync::Arc::new({}));{}.as_ref().unwrap().clone()}}
            }}}};",
            name.to_snake_case(),
            shared_field,
            shared_field,
            contents,
            shared_field
        );
        fn_gen.line(lazy_init);
    }

    fn_gen.line("let entity = world.create_entity_unchecked()");
    for tag in &def.tags {
        fn_gen.line(format!(".with(tag::{})", tag));
    }
    for (name, contents) in &def.components {
        let contents = stringify_component(&name, &contents);
        fn_gen.line(format!(".with({})", contents));
    }
    for (name, _) in &def.shared_components {
        fn_gen.line(format!(".with(component::Shared{}::from(shared_{}))", name, name.to_snake_case()));
    }
    fn_gen.line(&format!(".with(component::Reflection{{id:\"{}_{}\"}})", reflection_prefix, def.name));
    fn_gen.line(".build();");
    for line in buffers.1 {
        fn_gen.line(&line);
    }

    fn_gen.line("entity");
    fn_gen
}
