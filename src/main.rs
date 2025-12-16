#![allow(non_snake_case)]
#![windows_subsystem = "windows"]

mod components;
mod state;

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use std::time::Duration;

#[cfg(target_os = "windows")]
use tauri_winrt_notification::{Duration as ToastDuration, Scenario, Toast};

use crate::components::settings::SettingsModal;
use crate::components::timer_circle::TimerCircle;
use crate::state::{NotificationMode, TimerMode, TimerState};

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");

    let config = dioxus::desktop::Config::new()
        .with_custom_head(r#"<link rel="stylesheet" href="assets/style.css">"#.to_string())
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("Focus Timer")
                .with_resizable(true)
                .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(800.0, 600.0)),
        );

    LaunchBuilder::desktop().with_cfg(config).launch(App);
}

fn App() -> Element {
    let mut timer_state = use_signal(|| TimerState::new(25, 2));
    let mut show_settings = use_signal(|| false);
    let window = dioxus::desktop::use_window();

    // Timer Loop
    use_future(move || {
        let window = window.clone();
        async move {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();

            loop {
                tokio::select! {
                    _ = rx.recv() => {
                        window.set_focus();
                    }
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {
                        let finished = timer_state.write().tick();

                        if finished {
                             // Automatically switch mode logic
                             let current_mode = timer_state.read().mode;
                             let new_mode = match current_mode {
                                 TimerMode::Work => TimerMode::Pause,
                                 TimerMode::Pause => TimerMode::Work,
                             };
                             
                             // We update the state to the new mode so the UI (and next timer) reflects it
                             timer_state.write().switch_mode(new_mode);

                             let state = timer_state.read();
                             let title = match current_mode { // Use previous mode for the notification message
                                 TimerMode::Work => "Focus Session Complete",
                                 TimerMode::Pause => "Break Complete",
                             };
                             let body = match current_mode {
                                 TimerMode::Work => "Time to take a break!",
                                 TimerMode::Pause => "Time to focus!",
                             };
                             let mode = state.notification_mode;
            
                             match mode {
                                 NotificationMode::Popup => {
                                     window.set_focus();
                                 },
                                 NotificationMode::Notification | NotificationMode::NotificationPersistent => {
                                     // Show notification and wait for action in a separate blocking task
                                     let tx_clone = tx.clone();
                                     // We need to own the strings to move them into the thread
                                     let title = title.to_string();
                                     let body = body.to_string();

                                     tokio::task::spawn_blocking(move || {
                                         #[cfg(target_os = "windows")]
                                         {
                                             let mut toast = Toast::new(Toast::POWERSHELL_APP_ID)
                                                 .title(&title)
                                                 .text1(&body);

                                             if mode == NotificationMode::NotificationPersistent {
                                                  toast = toast.scenario(Scenario::Alarm);
                                                  toast = toast.add_button("Ok", "Ok");
                                             } else {
                                                  toast = toast.duration(ToastDuration::Short);
                                             }

                                             let _ = toast
                                                 .on_activated(move |_| {
                                                      let _ = tx_clone.send(());
                                                      Ok(())
                                                 })
                                                 .show();
                                         }

                                         #[cfg(not(target_os = "windows"))]
                                         {
                                             let timeout = if mode == NotificationMode::NotificationPersistent {
                                                 Timeout::Never
                                             } else {
                                                 Timeout::Default
                                             };

                                             let result = Notification::new()
                                                 .summary(&title)
                                                 .body(&body)
                                                 .timeout(timeout)
                                                 .action("default", "Focus")
                                                 .show();

                                             if let Ok(handle) = result {
                                                 handle.wait_for_action(|_action| {
                                                     let _ = tx_clone.send(());
                                                 });
                                             }
                                         }
                                     });
                                 }
                             }
                        }
                    }
                }
            }
        }
    });

    rsx! {
        div { class: "app-container",
            div { class: "timer-section",
                TimerCircle { state: timer_state }

                div { class: "controls", style: "margin-top: 30px; display: flex; gap: 10px;",
                    button { 
                        class: "btn",
                        onclick: move |_| timer_state.write().toggle(),
                        if timer_state.read().is_running { 
                            "Pause" 
                        } else { 
                            match timer_state.read().mode {
                                TimerMode::Work => "Start to Focus",
                                TimerMode::Pause => "Start Break",
                            }
                        }
                    }
                    button {
                        class: "btn-icon",
                        title: "Settings",
                        onclick: move |_| show_settings.set(true),
                        "⚙" // Simple gear icon for now
                    }
                    button {
                         class: "btn-icon",
                         title: "Reset",
                         onclick: move |_| timer_state.write().reset_current_mode(),
                         "↺"
                    }
                }

                div { style: "margin-top: 10px; color: var(--text-secondary);",
                    if timer_state.read().mode == TimerMode::Work { "Focus Mode" } else { "Break Mode" }
                }
            }

            div { class: "sidebar",
                div { class: "card",
                    h3 { "Focus Time of Today" }
                    p { class: "highlight-text", style: "color: #ef4444; font-size: 2em;", "0m" }
                }
                div { class: "card",
                    h3 { "Today" }
                    div { style: "text-align: center; color: var(--text-secondary);",
                        "No Task"
                    }
                }
            }

            if show_settings() {
                SettingsModal {
                    on_close: move |_| show_settings.set(false),
                    state: timer_state
                }
            }
        }
    }
}
