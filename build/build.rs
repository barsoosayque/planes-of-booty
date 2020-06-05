use crate::codegen::*;
use def::*;
use std::{fs, path::PathBuf};

mod codegen;
mod def;

macro_rules! read_from {
    ($path:expr => $type:ident) => {
        fs::read_dir($path).unwrap().filter_map(|r| r.ok()).filter_map(|entry| {
            if !entry.file_type().unwrap().is_file() {
                println!("Skipping {:?}, not a file.. ", entry.path());
                None
            } else {
                let path = entry.path();
                let content = fs::read_to_string(&path).unwrap();

                print!("Parsing file {:?} as $type.. ", path);
                match serde_yaml::from_str::<$type>(&content) {
                    Ok(def) => {
                        println!("Success !");
                        let name = path.file_stem().and_then(|s| s.to_str()).unwrap();
                        Some(($type { name: name.to_owned(), ..def }, path))
                    },
                    Err(err) => {
                        println!("Error !");
                        panic!("Error while parsing $type definition file {:?}: {}", path, err);
                    },
                }
            }
        })
    };
}
macro_rules! process_defs {
    ($from:expr => $to:expr, $type:ident, $gen:expr) => {
        let (defs, paths): (Vec<$type>, Vec<PathBuf>) = read_from!($from => $type).unzip();
        fs::write($to, $gen(&defs)).unwrap();
        println!("Total $type generated: {}", defs.len());
        for path in paths {
            println!("cargo:rerun-if-changed={:?}", path);
        }
    }
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

    process_defs!("resources/entities" => format!("{}/generated/entity.rs", out_dir), EntityDef, |entities|{
        generate_full_group(entities, "e").to_string()
    });

    process_defs!("resources/items" => format!("{}/generated/item.rs", out_dir), EntityDef, |items|{
        let mut items_body = generate_full_group(items, "i");
        items_body.raw(&generate_array_by_filter(items, "ANY_COMMON", is_def_common));
        items_body.raw(&generate_array_by_filter(items, "ANY_RARE", is_def_rare));
        items_body.raw(&generate_array_by_filter(items, "ANY_LEGENDARY", is_def_legendary));
        items_body.to_string()
    });

    process_defs!("resources/particles" => format!("{}/generated/particle.rs", out_dir), EntityDef, |particles|{
        generate_spawn_only(particles, "p").to_string()
    });

    process_defs!("resources/arenas" => format!("{}/generated/arena.rs", out_dir), ArenaDef, |arenas|{
        generate_arenas(arenas).to_string()
    });

    process_defs!("resources/spawn_groups" => format!("{}/generated/spawn_group.rs", out_dir), SpawnGroupDef, |spawn_groups|{
        generate_spawn_groups(spawn_groups).to_string()
    });
}
