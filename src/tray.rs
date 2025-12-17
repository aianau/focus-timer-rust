use tray_icon::{TrayIcon, TrayIconBuilder, Icon};
use tray_icon::menu::{Menu, MenuItem};
use usvg::{Tree, Options};
use tiny_skia::{Pixmap, Transform};

pub fn create_tray_icon() -> TrayIcon {
    let (icon_rgba, icon_width, icon_height) = load_icon_data();
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to create icon");
    
    // Create menu
    let menu = Menu::new();
    // We can't easily capture the ID to check it later without globals or passing it out, 
    // but we can check the text or use a predefined ID if possible.
    // tray-icon menu items have IDs. 
    // Let's rely on text or assume there is only one item for now, or just expose the menu item.
    let exit_item = MenuItem::new("Exit", true, None);
    let _ = menu.append(&exit_item);

    TrayIconBuilder::new()
        .with_tooltip("Focus Timer")
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .build()
        .unwrap()
}

pub fn load_icon_data() -> (Vec<u8>, u32, u32) {
    let svg_data = include_bytes!("../assets/timer-svgrepo-com.svg");
    let options = Options::default();
    let tree = Tree::from_data(svg_data, &options).expect("Failed to parse SVG");
    
    const WIDTH: u32 = 32;
    const HEIGHT: u32 = 32;
    
    let mut pixmap = Pixmap::new(WIDTH, HEIGHT).expect("Failed to create pixmap");
    
    let svg_width = tree.size().width();
    let svg_height = tree.size().height();
    let scale_x = WIDTH as f32 / svg_width;
    let scale_y = HEIGHT as f32 / svg_height;
    
    let transform = Transform::from_scale(scale_x, scale_y);
    
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    
    (pixmap.take(), WIDTH, HEIGHT)
}
