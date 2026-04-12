use crate::state::{NotificationMode, TimerState};
use dioxus::prelude::*;

#[component]
pub fn SettingsModal(on_close: EventHandler<()>, state: Signal<TimerState>) -> Element {
    // Local state for inputs to avoid updating global state on every keystroke if desired,
    // but binding directly is simpler for this scope.
    let mut s = state;

    let work_mins = s.read().work_duration.as_secs() / 60;
    let pause_mins = s.read().pause_duration.as_secs() / 60;
    let current_notification_mode = s.read().notification_mode;

    rsx! {
        div { class: "modal-overlay",
            div { class: "modal-content",
                h2 { style: "margin-top: 0;", "Settings" }

                div { class: "input-group",
                    label { "Work Duration (minutes)" }
                    input {
                        r#type: "number",
                        value: "{work_mins}",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<u64>() {
                                s.write().set_work_duration(val);
                            }
                        }
                    }
                }

                div { class: "input-group",
                    label { "Pause Duration (minutes)" }
                    input {
                        r#type: "number",
                        value: "{pause_mins}",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<u64>() {
                                s.write().set_pause_duration(val);
                            }
                        }
                    }
                }

                div { class: "input-group",
                    label { "Notification Style" }
                    select {
                        style: "width: 100%; padding: 12px; border-radius: 8px; border: 1px solid #444; background: #1e1e2e; color: white; font-size: 1rem;",
                        onchange: move |evt| {
                            let mode = match evt.value().as_str() {
                                "Notification" => NotificationMode::Notification,
                                "Persistent" => NotificationMode::NotificationPersistent,
                                _ => NotificationMode::Popup,
                            };
                            s.write().set_notification_mode(mode);
                        },
                        option { value: "Popup", selected: current_notification_mode == NotificationMode::Popup, "Popup Window" }
                        option { value: "Notification", selected: current_notification_mode == NotificationMode::Notification, "Notification" }
                        option { value: "Persistent", selected: current_notification_mode == NotificationMode::NotificationPersistent, "Persistent Notification" }
                    }
                }

                div { class: "input-group", style: "display: flex; align-items: center; gap: 10px;",
                    input {
                        r#type: "checkbox",
                        style: "width: auto;",
                        checked: s.read().hide_completed_tasks,
                        onchange: move |evt| {
                             s.write().set_hide_completed_tasks(evt.checked());
                        }
                    }
                    label { style: "margin: 0;", "Hide Completed Tasks" }
                }

                div { class: "input-group", style: "display: flex; align-items: center; gap: 10px;",
                    input {
                        r#type: "checkbox",
                        style: "width: auto;",
                        checked: s.read().auto_delete_old_tasks,
                        onchange: move |evt| {
                             s.write().set_auto_delete_old_tasks(evt.checked());
                        }
                    }
                    label { style: "margin: 0;", "Auto-delete completed tasks older than 48h" }
                }

                div { class: "input-group", style: "display: flex; align-items: center; gap: 10px;",
                    input {
                        r#type: "checkbox",
                        style: "width: auto;",
                        checked: s.read().run_at_startup,
                        onchange: move |evt| {
                             s.write().set_run_at_startup(evt.checked());
                        }
                    }
                    label { style: "margin: 0;", "Run at Startup" }
                }

                div { class: "input-group", style: "display: flex; align-items: center; gap: 10px;",
                    input {
                        r#type: "checkbox",
                        style: "width: auto;",
                        checked: s.read().show_start_menu_icon,
                        onchange: move |evt| {
                             s.write().set_show_start_menu_icon(evt.checked());
                        }
                    }
                    label { style: "margin: 0;", "Show Start Menu Icon" }
                }

                div { class: "input-group",
                    button {
                        class: "btn",
                        style: "width: auto; background-color: #555; color: white; margin-top: 10px; font-size: 0.8rem; padding: 6px 12px;",
                        onclick: move |_| {
                            s.write().remove_completed_tasks();
                        },
                        "Delete Completed Tasks"
                    }
                }

                div { style: "display: flex; justify-content: flex-end; margin-top: 20px;",
                    button {
                        class: "btn",
                        onclick: move |_| on_close.call(()),
                        "Close"
                    }
                }
            }
        }
    }
}
