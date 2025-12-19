use std::fmt;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::{PathBuf, Path};
use chrono::{DateTime, Local};
use dioxus_logger::tracing::{info, error};

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum TimerMode {
    Work,
    Pause,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum NotificationMode {
    Popup,
    Notification,
    NotificationPersistent,
}

impl fmt::Display for NotificationMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotificationMode::Popup => write!(f, "Popup Window"),
            NotificationMode::Notification => write!(f, "Notification"),
            NotificationMode::NotificationPersistent => write!(f, "Persistent Notification"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FocusSession {
    pub start_time: DateTime<Local>,
    pub duration_secs: u64,
    pub mode: TimerMode,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: u64,
    pub title: String,
    pub completed: bool,
    pub created_at: DateTime<Local>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct SessionHistory {
    pub sessions: Vec<FocusSession>,
    pub tasks: Vec<Task>,
}

impl SessionHistory {
    fn get_path() -> PathBuf {
        #[cfg(test)]
        return PathBuf::from("test_focus_history.json");
        #[cfg(not(test))]
        return PathBuf::from("focus_history.json");
    }

    pub fn load() -> Self {
        let path = Self::get_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                return serde_json::from_str(&content).unwrap_or_default();
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::get_path();
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, content);
        }
    }

    pub fn add_session(&mut self, duration: Duration, mode: TimerMode) {
        self.sessions.push(FocusSession {
            start_time: Local::now(),
            duration_secs: duration.as_secs(),
            mode,
        });
        self.save();
    }

    pub fn get_today_focus_duration(&self) -> Duration {
        let today = Local::now().date_naive();
        let total_secs: u64 = self.sessions.iter()
            .filter(|s| s.start_time.date_naive() == today && s.mode == TimerMode::Work)
            .map(|s| s.duration_secs)
            .sum();
        Duration::from_secs(total_secs)
    }

    pub fn get_today_break_duration(&self) -> Duration {
        let today = Local::now().date_naive();
        let total_secs: u64 = self.sessions.iter()
            .filter(|s| s.start_time.date_naive() == today && s.mode == TimerMode::Pause)
            .map(|s| s.duration_secs)
            .sum();
        Duration::from_secs(total_secs)
    }

    // Task management
    pub fn add_task(&mut self, title: String) {
        let id = self.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        self.tasks.push(Task {
            id,
            title,
            completed: false,
            created_at: Local::now(),
        });
        self.save();
    }

    pub fn toggle_task(&mut self, id: u64) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.completed = !task.completed;
            self.save();
        }
    }

    pub fn remove_task(&mut self, id: u64) {
        self.tasks.retain(|t| t.id != id);
        self.save();
    }

    pub fn remove_completed_tasks(&mut self) {
        self.tasks.retain(|t| !t.completed);
        self.save();
    }
    
    pub fn get_today_tasks(&self) -> Vec<Task> {
        let today = Local::now().date_naive();
        self.tasks.iter()
            .filter(|t| !t.completed || t.created_at.date_naive() == today)
            .cloned()
            .collect()
    }

    pub fn check_auto_delete(&mut self) {
        let threshold = Local::now() - chrono::Duration::hours(48);
        self.tasks.retain(|t| !t.completed || t.created_at >= threshold);
        self.save();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimerState {
    pub current_time: Duration,
    pub work_duration: Duration,
    pub pause_duration: Duration,
    pub is_running: bool,
    pub mode: TimerMode,
    pub notification_mode: NotificationMode,
    pub history: SessionHistory,
    pub hide_completed_tasks: bool,
    pub auto_delete_old_tasks: bool,
    pub run_at_startup: bool,
    pub window_width: u32,
    pub window_height: u32,
    pub overtime: Duration,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub work_minutes: u64,
    pub pause_minutes: u64,
    pub notification_mode: Option<NotificationMode>,
    pub hide_completed_tasks: Option<bool>,
    pub auto_delete_old_tasks: Option<bool>,
    pub run_at_startup: Option<bool>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            work_minutes: 25,
            pause_minutes: 5,
            notification_mode: Some(NotificationMode::NotificationPersistent),
            hide_completed_tasks: Some(false),
            auto_delete_old_tasks: Some(false),
            run_at_startup: Some(false),
            window_width: Some(800),
            window_height: Some(600),
        }
    }
}

impl AppConfig {
    pub fn get_config_path() -> PathBuf {
        #[cfg(test)]
        return PathBuf::from("test_focus_timer_config.json");
        #[cfg(not(test))]
        return PathBuf::from("focus_timer_config.json");
    }

    pub fn load() -> Option<Self> {
        let path = Self::get_config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                return serde_json::from_str(&content).ok();
            }
        }
        None
    }

    pub fn save(&self) {
        let path = Self::get_config_path();
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, content);
        }
    }
}

impl TimerState {
    pub fn new(default_work_minutes: u64, default_pause_minutes: u64) -> Self {
        // Try to load config, otherwise use defaults
        let (work_minutes, pause_minutes, notif_mode, hide_completed, auto_delete, run_startup, width, height) = if let Some(config) = AppConfig::load() {
            (
                config.work_minutes, 
                config.pause_minutes, 
                config.notification_mode.unwrap_or(NotificationMode::NotificationPersistent),
                config.hide_completed_tasks.unwrap_or(false),
                config.auto_delete_old_tasks.unwrap_or(false),
                config.run_at_startup.unwrap_or(false),
                config.window_width.unwrap_or(800),
                config.window_height.unwrap_or(600)
            )
        } else {
            (default_work_minutes, default_pause_minutes, NotificationMode::NotificationPersistent, false, false, false, 800, 600)
        };

        let work_duration = Duration::from_secs(work_minutes * 60);
        let pause_duration = Duration::from_secs(pause_minutes * 60);
        
        let mut state = Self {
            current_time: work_duration,
            work_duration,
            pause_duration,
            is_running: false,
            mode: TimerMode::Work,
            notification_mode: notif_mode,
            history: SessionHistory::load(),
            hide_completed_tasks: hide_completed,
            auto_delete_old_tasks: auto_delete,
            run_at_startup: run_startup,
            window_width: width,
            window_height: height,
            overtime: Duration::from_secs(0),
        };

        // Sync startup state with registry on startup to ensure consistency
        #[cfg(target_os = "windows")]
        {
            state.run_at_startup = state.check_startup_registry();
        }

        if state.auto_delete_old_tasks {
            state.history.check_auto_delete();
        }

        state
    }

    fn save_config(&self) {
        let config = AppConfig {
            work_minutes: self.work_duration.as_secs() / 60,
            pause_minutes: self.pause_duration.as_secs() / 60,
            notification_mode: Some(self.notification_mode),
            hide_completed_tasks: Some(self.hide_completed_tasks),
            auto_delete_old_tasks: Some(self.auto_delete_old_tasks),
            run_at_startup: Some(self.run_at_startup),
            window_width: Some(self.window_width),
            window_height: Some(self.window_height),
        };
        config.save();
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) {
        if self.window_width != width || self.window_height != height {
            self.window_width = width;
            self.window_height = height;
            self.save_config();
        }
    }

    /// Ticks the timer. Returns true if the timer just reached 00:00 (Session Completed).
    pub fn tick(&mut self) -> bool {
        if self.is_running {
            if self.current_time.as_secs() > 0 {
                self.current_time = self.current_time.saturating_sub(Duration::from_secs(1));
                if self.current_time.as_secs() == 0 {
                    // Timer just hit 0. Continue running for overtime.
                    return true;
                }
            } else {
                // In overtime
                self.overtime += Duration::from_secs(1);
            }
        }
        false
    }

    pub fn toggle(&mut self) {
        self.is_running = !self.is_running;
    }

    pub fn finish_current_session(&mut self) {
        let duration = match self.mode {
            TimerMode::Work => self.work_duration,
            TimerMode::Pause => self.pause_duration,
        };
        let actual_duration = duration + self.overtime;
        self.history.add_session(actual_duration, self.mode);

        let new_mode = match self.mode {
            TimerMode::Work => TimerMode::Pause,
            TimerMode::Pause => TimerMode::Work,
        };
        
        self.switch_mode(new_mode);
        self.overtime = Duration::from_secs(0);
    }

    pub fn switch_mode(&mut self, mode: TimerMode) {
        self.mode = mode;
        self.is_running = false;
        self.reset_current_mode();
    }

    pub fn reset_current_mode(&mut self) {
        self.current_time = match self.mode {
            TimerMode::Work => self.work_duration,
            TimerMode::Pause => self.pause_duration,
        };
        self.overtime = Duration::from_secs(0);
    }

    pub fn set_work_duration(&mut self, minutes: u64) {
        self.work_duration = Duration::from_secs(minutes * 60);
        if self.mode == TimerMode::Work && !self.is_running {
            self.current_time = self.work_duration;
        }
        self.save_config();
    }

    pub fn set_pause_duration(&mut self, minutes: u64) {
        self.pause_duration = Duration::from_secs(minutes * 60);
        if self.mode == TimerMode::Pause && !self.is_running {
            self.current_time = self.pause_duration;
        }
        self.save_config();
    }
    
    pub fn set_notification_mode(&mut self, mode: NotificationMode) {
        self.notification_mode = mode;
        self.save_config();
    }

    pub fn set_hide_completed_tasks(&mut self, hide: bool) {
        self.hide_completed_tasks = hide;
        self.save_config();
    }

    pub fn set_auto_delete_old_tasks(&mut self, auto_delete: bool) {
        self.auto_delete_old_tasks = auto_delete;
        if auto_delete {
            self.history.check_auto_delete();
        }
        self.save_config();
    }

    #[cfg(target_os = "windows")]
    fn check_startup_registry(&self) -> bool {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") {
            return key.get_value::<String, _>("FocusTimerRust").is_ok();
        }
        false
    }
    
    #[cfg(not(target_os = "windows"))]
    fn check_startup_registry(&self) -> bool {
        false
    }

    pub fn set_run_at_startup(&mut self, run: bool) {
        self.run_at_startup = run;
        
        #[cfg(target_os = "windows")]
        {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let path = Path::new("Software").join("Microsoft").join("Windows").join("CurrentVersion").join("Run");
            
            if run {
                if let Ok((key, _)) = hkcu.create_subkey(&path) {
                    if let Ok(exe_path) = std::env::current_exe() {
                         if let Some(parent) = exe_path.parent() {
                             let parent_str = parent.to_str().unwrap_or("");
                             let exe_name = exe_path.file_name().unwrap_or_default().to_str().unwrap_or("");
                             let cmd_val = format!("cmd /c start \"\" /d \"{}\" \"{}\"", parent_str, exe_name);
                             info!("Registry command: {}", cmd_val);
                             let _ = key.set_value("FocusTimerRust", &cmd_val);
                         }
                    }
                }
            } else {
                if let Err(e) = hkcu.open_subkey_with_flags(&path, KEY_SET_VALUE)
                    .and_then(|key| key.delete_value("FocusTimerRust")) {
                    error!("{}", e);
                }
            }
        }

        self.save_config();
    }

    pub fn remove_completed_tasks(&mut self) {
        self.history.remove_completed_tasks();
    }

    pub fn total_duration(&self) -> Duration {
        match self.mode {
            TimerMode::Work => self.work_duration,
            TimerMode::Pause => self.pause_duration,
        }
    }
    
    pub fn progress(&self) -> f32 {
        let total = self.total_duration().as_secs_f32();
        if total == 0.0 {
            return 0.0;
        }
        // Progress usually means how much time has passed, or how much is left. 
        // Circular timers often deplete. Let's return fraction remaining (0.0 to 1.0).
        self.current_time.as_secs_f32() / total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_initialization() {
        let state = TimerState::new(25, 5);
        assert_eq!(state.work_duration, Duration::from_secs(25 * 60));
        assert_eq!(state.pause_duration, Duration::from_secs(5 * 60));
        assert_eq!(state.mode, TimerMode::Work);
        assert!(!state.is_running);
    }

    #[test]
    fn test_timer_tick() {
        let mut state = TimerState::new(25, 5);
        state.toggle(); // Start timer
        assert!(state.is_running);

        let initial_time = state.current_time;
        state.tick();
        assert_eq!(state.current_time, initial_time - Duration::from_secs(1));
    }

    #[test]
    fn test_timer_completion() {
        let mut state = TimerState::new(1, 1);
        state.current_time = Duration::from_secs(1);
        state.toggle();
        
        // Tick to 0
        let completed = state.tick();
        assert!(completed);
        assert_eq!(state.current_time, Duration::from_secs(0));

        // Tick past 0 (overtime)
        let completed_overtime = state.tick();
        assert!(!completed_overtime);
        assert_eq!(state.overtime, Duration::from_secs(1));
    }

    #[test]
    fn test_switch_mode() {
        let mut state = TimerState::new(25, 5);
        state.switch_mode(TimerMode::Pause);
        
        assert_eq!(state.mode, TimerMode::Pause);
        assert_eq!(state.current_time, state.pause_duration);
        assert!(!state.is_running);
    }

    #[test]
    fn test_task_management() {
        let mut history = SessionHistory::default();
        
        history.add_task("Test Task".to_string());
        assert_eq!(history.tasks.len(), 1);
        assert_eq!(history.tasks[0].title, "Test Task");
        assert!(!history.tasks[0].completed);

        let id = history.tasks[0].id;
        history.toggle_task(id);
        assert!(history.tasks[0].completed);

        history.remove_task(id);
        assert_eq!(history.tasks.len(), 0);
    }
}
