use crate::def::{ComponentDef, EntityDef, PartValue, ShapeshifterFormDef};
use codegen::*;
use heck::{CamelCase, ShoutySnakeCase, SnakeCase};
use itertools::Itertools;
use std::collections::{BTreeMap as Map, BTreeSet as Set};

pub fn generate_full_group(defs: &Vec<EntityDef>, group_name: &str) -> Scope {
    let mut scope = Scope::new();
    scope.raw(&generate_names_enum(defs));
    scope.push_fn(generate_generic_spawn_fn(&defs));
    scope.push_fn(generate_generic_view_fn(&defs));
    for def in defs {
        scope.push_fn(generate_spawn_fn(&def, group_name));
    }
    scope
}

pub fn generate_spawn_only(defs: &Vec<EntityDef>, group_name: &str) -> Scope {
    let mut scope = Scope::new();
    scope.raw(&generate_names_enum(defs));
    scope.push_fn(generate_generic_spawn_fn(&defs));
    for def in defs {
        scope.push_fn(generate_spawn_fn(&def, group_name));
    }
    scope
}

pub fn generate_names_enum(defs: &Vec<EntityDef>) -> String {
    let names = defs.into_iter().map(|def| def.name.to_camel_case());
    let r#enum = format!(
        "#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)] pub enum ID{{{}}}",
        names.clone().join(",")
    );
    let array = format!("pub const IDS: [ID; {}] = [{}];", names.len(), names.map(|n| format!("ID::{}", n)).join(","));
    format!("{}\n{}", r#enum, array)
}

pub fn generate_array_by_filter<F: FnMut(&&EntityDef) -> bool>(
    defs: &Vec<EntityDef>,
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

pub fn generate_generic_spawn_fn(defs: &Vec<EntityDef>) -> Function {
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

pub fn generate_generic_view_fn(defs: &Vec<EntityDef>) -> Function {
    let mut fn_gen = Function::new("view");
    fn_gen.arg("id", "ID").arg("ctx", "&mut ggez::Context").arg("assets", "&mut crate::assets::AssetManager");
    fn_gen.ret("Option<(std::sync::Arc<crate::assets::ImageAsset>, crate::math::Size2f)>");
    fn_gen.vis("pub");

    fn_gen.line("match id {");
    for def in defs {
        let name = def.name.to_camel_case();
        match get_view_from(&def.name, &def.components, "Sprite", "asset")
            .or(def.shapeshifter_forms.first().and_then(|f| get_view_from(&def.name, &f.components, "Sprite", "asset")))
        {
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
        PartValue::Collide { shape, .. } => collect_init_and_fin(shape, buffers),
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
fn shapeshifter_impl(form: &ShapeshifterFormDef) -> (String, String) {
    let struct_name = format!("ShapeshifterForm{}", form.form.to_camel_case());
    let r#struct = format!("struct {};", struct_name);
    let on_begin_body = form
        .components
        .iter()
        .map(|(name, def)| format!("update.insert(e, {});", stringify_component(name, def)))
        .collect::<Vec<_>>()
        .join("\n");
    let on_end_body = form
        .components
        .iter()
        .map(|(name, _)| format!("update.remove::<component::{}>(e);", name))
        .collect::<Vec<_>>()
        .join("\n");
    let can_update_body = form
        .conditions
        .iter()
        .map(|(component, conditions)| {
            let conditions = conditions.iter().map(|(field, condition)| format!("c.{}{}", field, condition)).join("&&");
            format!("{{world.read_storage::<component::{}>().get(e).map(|c| {}).unwrap_or(true)}}", component, conditions)
        })
        .collect::<Vec<_>>()
        .join("&&");
    let r#impl = format!(
        "impl component::ShapeshifterForm for {} {{\n\
            fn can_update<'a>(&self, e: specs::Entity, world: &specs::World) -> bool{{{}}}\n\
            fn on_begin<'a>(&self, e: specs::Entity, update: &specs::LazyUpdate, (assets, ctx): component::ShapeshifterFormData<'a>){{{}}}\n\
            fn on_end<'a>(&self, e: specs::Entity, update: &specs::LazyUpdate, (assets, ctx): component::ShapeshifterFormData<'a>){{{}}}\n\
            fn time(&self) -> f32 {{{}f32}}\n\
        }}",
        struct_name, if can_update_body.is_empty() { "true" } else { &can_update_body }, on_begin_body, on_end_body, form.time
    );
    (struct_name, format!("{}\n{}", r#struct, r#impl))
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

    if !def.shapeshifter_forms.is_empty() {
        let mut forms: Vec<String> = vec![];
        for form in &def.shapeshifter_forms {
            let (struct_name, definition) = shapeshifter_impl(&form);
            let var_name = struct_name.to_shouty_snake_case();
            fn_gen.line(definition);
            fn_gen.line(format!("const {}: {} = {};", var_name, struct_name, struct_name));
            forms.push(format!("&{}", var_name));
        }
        fn_gen.line(format!(
            "const SHAPESHIFTER_FORMS: [&'static dyn component::ShapeshifterForm; {}] = [{}];",
            forms.len(),
            forms.join(",")
        ));
    }

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
    if !def.shapeshifter_forms.is_empty() {
        // use first shape
        for (name, contents) in &def.shapeshifter_forms.first().unwrap().components {
            let contents = stringify_component(&name, &contents);
            fn_gen.line(format!(".with({})", contents));
        }
        fn_gen.line(".with(component::Shapeshifter{forms:&SHAPESHIFTER_FORMS,current:0,time:0.0})");
    }
    fn_gen.line(&format!(".with(component::Reflection{{id:\"{}_{}\"}})", reflection_prefix, def.name));
    fn_gen.line(".build();");
    for line in buffers.1 {
        fn_gen.line(&line);
    }

    fn_gen.line("entity");
    fn_gen
}
