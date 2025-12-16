
use tray_icon::{TrayIcon, TrayIconBuilder, Icon};
use tray_icon::menu::{Menu, MenuItem};

pub fn create_tray_icon() -> TrayIcon {
    let icon = generate_icon();
    
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

fn generate_icon() -> Icon {
    const WIDTH: u32 = 16;
    const HEIGHT: u32 = 16;
    let mut rgba = Vec::new();
    for _ in 0..HEIGHT {
        for _ in 0..WIDTH {
            rgba.extend_from_slice(&[255, 0, 0, 255]); // Red
        }
    }
    Icon::from_rgba(rgba, WIDTH, HEIGHT).expect("Failed to create icon")
}

