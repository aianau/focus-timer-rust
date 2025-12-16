use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir).ancestors().nth(3).unwrap();
    let assets_src = Path::new("assets");
    let assets_dest = target_dir.join("assets");

    if assets_src.exists() {
        if !assets_dest.exists() {
            fs::create_dir(&assets_dest).unwrap();
        }

        for entry in fs::read_dir(assets_src).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap();
                let dest_path = assets_dest.join(file_name);
                fs::copy(&path, &dest_path).unwrap();
            }
        }
    }
    
    println!("cargo:rerun-if-changed=assets");
}

