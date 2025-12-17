use dioxus::prelude::*;
use dioxus::desktop::tao::window::ResizeDirection;

#[component]
pub fn ResizeHandles() -> Element {
    let window = dioxus::desktop::use_window();
    let w_n = window.clone();
    let w_e = window.clone();
    let w_s = window.clone();
    let w_w = window.clone();
    let w_nw = window.clone();
    let w_ne = window.clone();
    let w_se = window.clone();
    let w_sw = window.clone();

    rsx! {
        // Top
        div { 
            class: "resize-handle-n",
            onmousedown: move |_| { let _ = w_n.drag_resize_window(ResizeDirection::North); }
        }
        // Right
        div { 
            class: "resize-handle-e",
            onmousedown: move |_| { let _ = w_e.drag_resize_window(ResizeDirection::East); }
        }
        // Bottom
        div { 
            class: "resize-handle-s",
            onmousedown: move |_| { let _ = w_s.drag_resize_window(ResizeDirection::South); }
        }
        // Left
        div { 
            class: "resize-handle-w",
            onmousedown: move |_| { let _ = w_w.drag_resize_window(ResizeDirection::West); }
        }
        // Top-Left
        div { 
            class: "resize-handle-nw",
            onmousedown: move |_| { let _ = w_nw.drag_resize_window(ResizeDirection::NorthWest); }
        }
        // Top-Right
        div { 
            class: "resize-handle-ne",
            onmousedown: move |_| { let _ = w_ne.drag_resize_window(ResizeDirection::NorthEast); }
        }
        // Bottom-Right
        div { 
            class: "resize-handle-se",
            onmousedown: move |_| { let _ = w_se.drag_resize_window(ResizeDirection::SouthEast); }
        }
        // Bottom-Left
        div { 
            class: "resize-handle-sw",
            onmousedown: move |_| { let _ = w_sw.drag_resize_window(ResizeDirection::SouthWest); }
        }
    }
}

