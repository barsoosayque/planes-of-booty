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

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    fs::create_dir_all(format!("{}/generated/", out_dir)).unwrap();

    let (entities, entities_path): (Vec<EntityDef>, Vec<PathBuf>) = read_defs_from("resources/entities").unzip();
    fs::write(format!("{}/generated/entity.rs", out_dir), codegen::generate_full_group(&entities, "i")).unwrap();
    println!("Total entities generated: {}", entities.len());

    let (items, items_path): (Vec<EntityDef>, Vec<PathBuf>) = read_defs_from("resources/items").unzip();
    fs::write(format!("{}/generated/item.rs", out_dir), codegen::generate_full_group(&items, "i")).unwrap();
    println!("Total items generated: {}", entities.len());

    for path in entities_path {
        println!("cargo:rerun-if-changed={:?}", path);
    }
    for path in items_path {
        println!("cargo:rerun-if-changed={:?}", path);
    }
}
