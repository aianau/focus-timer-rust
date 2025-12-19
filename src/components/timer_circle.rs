use crate::state::TimerState;
use dioxus::prelude::*;
use std::f32::consts::PI;

#[component]
pub fn TimerCircle(state: Signal<TimerState>) -> Element {
    let s = state.read();
    let (minutes, seconds, is_overtime) = if s.current_time.as_secs() > 0 {
        let total_seconds = s.current_time.as_secs();
        (total_seconds / 60, total_seconds % 60, false)
    } else {
        let total_seconds = s.overtime.as_secs();
        (total_seconds / 60, total_seconds % 60, true)
    };
    
    let time_str = if is_overtime {
        format!("+{:02}:{:02}", minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    };

    let radius = 140.0;
    let stroke = 10.0;
    let normalized_radius = radius - stroke / 2.0;
    let circumference = normalized_radius * 2.0 * PI;

    // progress 1.0 -> full circle, 0.0 -> empty
    // s.progress() returns fraction remaining (1.0 down to 0.0)
    // In overtime, let's pulse or keep full?
    let progress = if is_overtime {
        1.0 // Full circle or maybe 0?
    } else {
        s.progress()
    };
    let stroke_dashoffset = circumference - (progress * circumference);

    let text_color = if is_overtime { "#e8f075" } else { "inherit" };

    rsx! {
        div {
            class: "timer-circle-container",
            style: "position: relative; width: {radius * 2.0}px; height: {radius * 2.0}px; display: flex; justify-content: center; align-items: center;",

            svg {
                height: "{radius * 2.0}",
                width: "{radius * 2.0}",
                style: "transform: rotate(-90deg);", // Start from top

                // Background Circle
                circle {
                    stroke: "#333",
                    stroke_width: "{stroke}",
                    fill: "transparent",
                    r: "{normalized_radius}",
                    cx: "{radius}",
                    cy: "{radius}"
                }

                // Progress Circle
                circle {
                    stroke: if is_overtime { "#e8f075" } else { "var(--accent-color)" },
                    stroke_dasharray: "{circumference} {circumference}",
                    style: "stroke-dashoffset: {stroke_dashoffset}",
                    stroke_width: "{stroke}",
                    stroke_linecap: "round",
                    fill: "transparent",
                    r: "{normalized_radius}",
                    cx: "{radius}",
                    cy: "{radius}",
                    // Transition for smooth movement
                    "transition": "stroke-dashoffset 0.5s linear"
                }
            }

            // Digital Time Display
            div {
                class: "timer-text",
                style: "position: absolute; font-size: 4rem; font-weight: 300; color: {text_color};",
                "{time_str}"
            }
        }
    }
}
