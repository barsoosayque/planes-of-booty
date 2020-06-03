use crate::codegen::*;
use def::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

mod codegen;
mod def;

fn read_defs_from<P: AsRef<Path>>(path: P) -> impl Iterator<Item = (EntityDef, PathBuf)> {
    fs::read_dir(path).unwrap().filter_map(|r| r.ok()).filter_map(|entry| {
        if !entry.file_type().unwrap().is_file() {
            println!("Skipping {:?}, not a file.. ", entry.path());
            None
        } else {
            let path = entry.path();
            let content = fs::read_to_string(&path).unwrap();

            print!("Parsing file {:?} as EntityDef.. ", path);
            match serde_yaml::from_str::<EntityDef>(&content) {
                Ok(def) => {
                    println!("Success !");
                    let name = path.file_stem().and_then(|s| s.to_str()).unwrap();
                    Some((EntityDef { name: name.to_owned(), ..def }, path))
                },
                Err(err) => {
                    println!("Error !");
                    panic!("Error while parsing EntityDef definition file {:?}: {}", path, err);
                },
            }
        }
    })
}
fn read_arenas_from<P: AsRef<Path>>(path: P) -> impl Iterator<Item = (ArenaDef, PathBuf)> {
    fs::read_dir(path).unwrap().filter_map(|r| r.ok()).filter_map(|entry| {
        if !entry.file_type().unwrap().is_file() {
            println!("Skipping {:?}, not a file.. ", entry.path());
            None
        } else {
            let path = entry.path();
            let content = fs::read_to_string(&path).unwrap();

            print!("Parsing file {:?} as ArenaDef.. ", path);
            match serde_yaml::from_str::<ArenaDef>(&content) {
                Ok(def) => {
                    println!("Success !");
                    let name = path.file_stem().and_then(|s| s.to_str()).unwrap();
                    Some((ArenaDef { name: name.to_owned(), ..def }, path))
                },
                Err(err) => {
                    println!("Error !");
                    panic!("Error while parsing ArenaDef definition file {:?}: {}", path, err);
                },
            }
        }
    })
}

fn filter_by_rarity(def: &&EntityDef, rarity: &str) -> bool {
    def.components.iter().any(|(name, contents)| {
        name == "Quality"
            && contents.parts.iter().any(|(part_name, part_value)| match part_value {
                PartValue::Rarity(vrarity)
                    if (part_name == "rarity"
                        && vrarity.to_lowercase().matches(&rarity.to_lowercase()).next().is_some()) =>
                {
                    true
                },
                _ => false,
            })
    })
}
fn is_def_common(def: &&EntityDef) -> bool { filter_by_rarity(def, "common") }
fn is_def_rare(def: &&EntityDef) -> bool { filter_by_rarity(def, "rare") }
fn is_def_legendary(def: &&EntityDef) -> bool { filter_by_rarity(def, "legendary") }

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    fs::create_dir_all(format!("{}/generated/", out_dir)).unwrap();

    let (entities, entities_path): (Vec<EntityDef>, Vec<PathBuf>) = read_defs_from("resources/entities").unzip();
    fs::write(format!("{}/generated/entity.rs", out_dir), generate_full_group(&entities, "e").to_string()).unwrap();
    println!("Total entities generated: {}", entities.len());

    let (items, items_path): (Vec<EntityDef>, Vec<PathBuf>) = read_defs_from("resources/items").unzip();
    let mut items_body = generate_full_group(&items, "i");
    items_body.raw(&generate_array_by_filter(&items, "ANY_COMMON", is_def_common));
    items_body.raw(&generate_array_by_filter(&items, "ANY_RARE", is_def_rare));
    items_body.raw(&generate_array_by_filter(&items, "ANY_LEGENDARY", is_def_legendary));
    fs::write(format!("{}/generated/item.rs", out_dir), items_body.to_string()).unwrap();
    println!("Total items generated: {}", items.len());

    let (particles, particles_path): (Vec<EntityDef>, Vec<PathBuf>) = read_defs_from("resources/particles").unzip();
    fs::write(format!("{}/generated/particle.rs", out_dir), generate_spawn_only(&particles, "p").to_string()).unwrap();
    println!("Total particles generated: {}", particles.len());

    let (arenas, arenas_path): (Vec<ArenaDef>, Vec<PathBuf>) = read_arenas_from("resources/arenas").unzip();
    fs::write(format!("{}/generated/arena.rs", out_dir), generate_arenas(&arenas).to_string()).unwrap();
    println!("Total arenas generated: {}", arenas.len());

    for path in entities_path {
        println!("cargo:rerun-if-changed={:?}", path);
    }
    for path in items_path {
        println!("cargo:rerun-if-changed={:?}", path);
    }
    for path in particles_path {
        println!("cargo:rerun-if-changed={:?}", path);
    }
    for path in arenas_path {
        println!("cargo:rerun-if-changed={:?}", path);
    }
}
