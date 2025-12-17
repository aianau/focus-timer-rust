#![windows_subsystem = "windows"]

mod components;
mod state;
mod tray;
mod events; // Add events module

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use std::time::Duration;
use tray_icon::{TrayIconEvent, menu::MenuEvent};

#[cfg(target_os = "windows")]
use tauri_winrt_notification::{Duration as ToastDuration, Scenario, Toast};

use crate::components::settings::SettingsModal;
use crate::components::timer_circle::TimerCircle;
use crate::components::titlebar::TitleBar;
use crate::components::task_list::TaskList; // Import TaskList
use crate::components::resize_handles::ResizeHandles;
use crate::state::{NotificationMode, TimerMode, TimerState, AppConfig};
use crate::events::AppEvent; // Import AppEvent

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");

    // Initialize tray icon (keep it alive)
    let _tray = tray::create_tray_icon();

    let (width, height) = if let Some(config) = AppConfig::load() {
        (config.window_width.unwrap_or(800) as f64, config.window_height.unwrap_or(600) as f64)
    } else {
        (800.0, 600.0)
    };

    // Load icon for window
    let (icon_rgba, icon_width, icon_height) = tray::load_icon_data();
    let window_icon = dioxus::desktop::tao::window::Icon::from_rgba(icon_rgba, icon_width, icon_height)
        .expect("Failed to create window icon");

    let config = dioxus::desktop::Config::new()
        .with_custom_head(r#"<link rel="stylesheet" href="assets/style.css">"#.to_string())
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("Focus Timer")
                .with_decorations(false) // Custom title bar for tray support
                .with_resizable(true)
                .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(width, height))
                .with_window_icon(Some(window_icon)),
        );

    LaunchBuilder::desktop().with_cfg(config).launch(app);
}

fn app() -> Element {
    let mut timer_state = use_signal(|| TimerState::new(25, 2));
    let mut show_settings = use_signal(|| false);
    let window = dioxus::desktop::use_window();
    let window_tray = window.clone();
    let window_timer = window.clone();
    let window_resize = window.clone();

    // Window Resize Observer
    use_future(move || {
        let window = window_resize.clone();
        let mut timer_state = timer_state;
        async move {
            loop {
                tokio::time::sleep(Duration::from_secs(2)).await;
                let size = window.inner_size();
                let scale = window.scale_factor();
                
                // Convert to logical size
                let width = (size.width as f64 / scale) as u32;
                let height = (size.height as f64 / scale) as u32;
                
                // Read current state to avoid unnecessary writes
                let (current_width, current_height) = {
                    let s = timer_state.read();
                    (s.window_width, s.window_height)
                };
                
                if current_width != width || current_height != height {
                     timer_state.write().set_window_size(width, height);
                }
            }
        }
    });

    // Listen to tray events
    use_future(move || {
        let window = window_tray.clone();
        async move {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            
            // Spawn a thread to forward tray events to async channel
            std::thread::spawn(move || {
                let tray_rx = TrayIconEvent::receiver();
                let menu_rx = MenuEvent::receiver();
                
                // We need to poll both. Or select.
                // Since these are crossbeam channels (implied by receiver()), we can use `select!`.
                // However, Dioxus/Tokio world is async.
                // Let's create a loop that checks both non-blocking or simple loop with sleep? 
                // Or better, move both to the same tx if we wrap them in an enum.
                
                // Simplified approach: loop and try_recv both
                loop {
                    if let Ok(event) = tray_rx.try_recv() {
                         let _ = tx.send(AppEvent::Tray(event));
                    }
                    if let Ok(event) = menu_rx.try_recv() {
                         let _ = tx.send(AppEvent::Menu(event));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            });

            while let Some(app_event) = rx.recv().await {
                 match app_event {
                     AppEvent::Tray(event) => {
                         if let TrayIconEvent::Click { button, .. } = event {
                             if button == tray_icon::MouseButton::Left {
                                 window.set_visible(true);
                                 window.set_focus();
                             }
                         }
                     }
                     AppEvent::Menu(_event) => {
                         // We assume it is Exit because it is the only item
                         std::process::exit(0);
                     }
                 }
            }
        }
    });

    // Timer Loop
    use_future(move || {
        let window = window_timer.clone();
        async move {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();

            loop {
                tokio::select! {
                    _ = rx.recv() => {
                        window.set_visible(true);
                        window.set_focus();
                    }
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {
                        let finished = timer_state.write().tick();

                        if finished {
                             // Automatically switch mode logic
                             let current_mode = timer_state.read().mode;

                             match current_mode {
                                 TimerMode::Work => {
                                     let duration = timer_state.read().work_duration;
                                     timer_state.write().history.add_session(duration, TimerMode::Work);
                                 }
                                 TimerMode::Pause => {
                                     let duration = timer_state.read().pause_duration;
                                     timer_state.write().history.add_session(duration, TimerMode::Pause);
                                 }
                             }

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
                                             // Notification logic for other OSs
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
        ResizeHandles {}
        TitleBar {}
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
                    p { class: "highlight-text", style: "color:rgb(164, 248, 86); font-size: 2em;", 
                        "{timer_state.read().history.get_today_focus_duration().as_secs() / 60}m" 
                    }
                    div { style: "margin-top: 10px; border-top: 1px solid #eee; padding-top: 10px;",
                        p { style: "color: var(--text-secondary); margin: 0;", "Break Time Today" }
                        p { style: "font-size: 1.2em; font-weight: bold; margin: 0;", 
                            "{timer_state.read().history.get_today_break_duration().as_secs() / 60}m" 
                        }
                    }
                }
                div { class: "card",
                    h3 { "Today" }
                    TaskList {
                        tasks: {
                            let tasks = timer_state.read().history.get_today_tasks();
                            if timer_state.read().hide_completed_tasks {
                                tasks.into_iter().filter(|t| !t.completed).collect()
                            } else {
                                tasks
                            }
                        },
                        on_add: move |title| timer_state.write().history.add_task(title),
                        on_toggle: move |id| timer_state.write().history.toggle_task(id),
                        on_remove: move |id| timer_state.write().history.remove_task(id),
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
