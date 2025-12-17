use dioxus::prelude::*;
use crate::state::Task;

#[component]
pub fn TaskList(tasks: Vec<Task>, on_add: EventHandler<String>, on_toggle: EventHandler<u64>, on_remove: EventHandler<u64>) -> Element {
    let mut new_task_title = use_signal(String::new);

    rsx! {
        div { class: "task-list",
            div { class: "task-input-container",
                input {
                    class: "task-input",
                    placeholder: "Add a task...",
                    value: "{new_task_title}",
                    oninput: move |evt| new_task_title.set(evt.value()),
                    onkeypress: move |evt| {
                        if evt.key() == Key::Enter && !new_task_title.read().trim().is_empty() {
                            on_add.call(new_task_title.read().clone());
                            new_task_title.set(String::new());
                        }
                    }
                }
                button {
                    class: "btn-icon-small",
                    onclick: move |_| {
                         if !new_task_title.read().trim().is_empty() {
                            on_add.call(new_task_title.read().clone());
                            new_task_title.set(String::new());
                        }
                    },
                    "+"
                }
            }
            div { class: "tasks",
                if tasks.is_empty() {
                     div { style: "text-align: center; color: var(--text-secondary); margin-top: 10px;", "No Tasks" }
                } else {
                    for task in tasks {
                        div { class: "task-item", key: "{task.id}",
                            input {
                                r#type: "checkbox",
                                checked: task.completed,
                                onchange: move |_| on_toggle.call(task.id)
                            }
                            span { 
                                class: if task.completed { "task-title completed" } else { "task-title" },
                                "{task.title}" 
                            }
                            button {
                                class: "btn-icon-small remove",
                                onclick: move |_| on_remove.call(task.id),
                                "×"
                            }
                        }
                    }
                }
            }
        }
    }
}

