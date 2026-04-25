use dioxus_logger::tracing::{info, error};
use std::process::Command;
use self_update::cargo_crate_version;

#[cfg(target_os = "windows")]
use tauri_winrt_notification::{Duration as ToastDuration, Scenario, Toast};

pub fn check_and_prompt_update() {
    std::thread::spawn(|| {
        info!("Checking for updates...");
        let updater = match self_update::backends::github::Update::configure()
            .repo_owner("aianau")
            .repo_name("focus-timer-rust")
            .bin_name("focus-timer-rust.exe")
            .current_version(cargo_crate_version!())
            .build()
        {
            Ok(u) => u,
            Err(e) => {
                error!("Failed to configure updater: {}", e);
                return;
            }
        };

        let latest_release = match updater.get_latest_release() {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to get latest release: {}", e);
                return;
            }
        };

        let is_greater = self_update::version::bump_is_greater(cargo_crate_version!(), &latest_release.version).unwrap_or(false);

        if is_greater {
            info!("Update available: {}", latest_release.version);
            prompt_user_for_update(&latest_release.version);
        } else {
            info!("App is up to date.");
        }
    });
}

#[cfg(target_os = "windows")]
fn prompt_user_for_update(new_version: &str) {
    let title = "Focus Timer Update Available";
    let body = format!("Version {} is available. Click to install.", new_version);

    let mut toast = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(title)
        .text1(&body)
        .scenario(Scenario::Reminder)
        .duration(ToastDuration::Long)
        .add_button("Update Now", "update");

    toast = toast.on_activated(|action| {
        if let Some(args) = action {
            if args == "update" {
                if let Ok(exe_path) = std::env::current_exe() {
                    let exe_str = exe_path.to_str().unwrap_or("");
                    // Request UAC elevation to perform the update
                    let ps_script = format!(
                        "Start-Process -FilePath '{}' -ArgumentList '--update' -Verb RunAs",
                        exe_str
                    );
                    let _ = Command::new("powershell")
                        .args(["-WindowStyle", "Hidden", "-NoProfile", "-Command", &ps_script])
                        .spawn();
                    std::process::exit(0);
                }
            }
        }
        Ok(())
    });

    if let Err(e) = toast.show() {
        error!("Failed to show update notification: {}", e);
    }
}

#[cfg(not(target_os = "windows"))]
fn prompt_user_for_update(new_version: &str) {
    info!("Update available: {}", new_version);
    // macOS/Linux native prompts can be implemented here if needed.
}

pub fn apply_update() {
    info!("Applying update...");
    match self_update::backends::github::Update::configure()
        .repo_owner("aianau")
        .repo_name("focus-timer-rust")
        .bin_name("focus-timer-rust.exe")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()
        .and_then(|u| u.update())
    {
        Ok(status) => {
            info!("Update successful: {}", status.version());
            // Restart the app normally
            if let Ok(exe_path) = std::env::current_exe() {
                let _ = Command::new(exe_path).spawn();
            }
        }
        Err(e) => {
            error!("Update failed: {}", e);
            // Optionally could show a failure notification here
        }
    }
    std::process::exit(0);
}
