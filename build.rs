use std::env;
use std::fs;
use std::path::Path;

#[cfg(windows)]
use tiny_skia::{Pixmap, Transform};
#[cfg(windows)]
use usvg::{Tree, Options};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir).ancestors().nth(3).unwrap();
    let assets_src = Path::new("assets");
    let assets_dest = target_dir.join("assets");

    if assets_src.exists() {
        if !assets_dest.exists() {
            fs::create_dir_all(&assets_dest).unwrap();
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

    #[cfg(windows)]
    embed_icon(&out_dir);
}

#[cfg(windows)]
fn embed_icon(out_dir: &str) {
    let icon_path = Path::new("assets/timer-svgrepo-com.svg");
    if !icon_path.exists() {
        return;
    }
    
    println!("cargo:rerun-if-changed=assets/timer-svgrepo-com.svg");

    // 1. Render SVG to Pixmap (256x256)
    let svg_data = fs::read(icon_path).expect("Failed to read SVG");
    let options = Options::default();
    let tree = Tree::from_data(&svg_data, &options).expect("Failed to parse SVG");
    
    let width = 256;
    let height = 256;
    let mut pixmap = Pixmap::new(width, height).expect("Failed to create pixmap");
    
    let svg_width = tree.size().width();
    let svg_height = tree.size().height();
    let scale_x = width as f32 / svg_width;
    let scale_y = height as f32 / svg_height;
    
    let transform = Transform::from_scale(scale_x, scale_y);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    
    // 2. Save as ICO using image crate
    // image crate expects Rgba8
    let img = image::RgbaImage::from_raw(width, height, pixmap.data().to_vec())
        .expect("Failed to create image buffer");
    
    let icon_dest = Path::new(out_dir).join("icon.ico");
    img.save_with_format(&icon_dest, image::ImageFormat::Ico)
        .expect("Failed to save ICO");

    // 3. Create .rc file and compile
    // We point to the absolute path of the generated ico file to be safe
    let icon_path_str = icon_dest.to_str().unwrap().replace("\\", "/");
    let rc_path = Path::new(out_dir).join("icon.rc");
    
    fs::write(&rc_path, format!(r#"id ICON "{}""#, icon_path_str)).expect("Failed to write .rc file");
    
    embed_resource::compile(rc_path.to_str().unwrap(), embed_resource::NONE);
}
