use dioxus::prelude::*;

#[component]
pub fn TitleBar() -> Element {
    let window = dioxus::desktop::use_window();

    let window_drag = window.clone();
    let window_min = window.clone();
    let window_close = window.clone();

    rsx! {
        div {
            class: "titlebar",
            // Drag region
            onmousedown: move |_| window_drag.drag(),
            
            div { class: "title", "Focus Timer" }
            
            div { class: "window-controls",
                // Prevent drag on buttons (stop propagation handled by default on buttons usually? no, explicit stop needed if parent has handler)
                onmousedown: move |e| e.stop_propagation(),
                
                button { 
                    class: "control-btn minimize", 
                    onclick: move |_| window_min.set_visible(false),
                    "─" 
                }
                button { 
                    class: "control-btn close", 
                    onclick: move |_| window_close.close(),
                    "✕" 
                }
            }
        }
    }
}
