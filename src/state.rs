use std::fmt;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Local};

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
    // ... existing methods ...
    fn get_path() -> PathBuf {
        PathBuf::from("focus_history.json")
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
    
    pub fn get_today_tasks(&self) -> Vec<Task> {
        let today = Local::now().date_naive();
        self.tasks.iter()
            .filter(|t| !t.completed || t.created_at.date_naive() == today)
            .cloned()
            .collect()
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
}

#[derive(Serialize, Deserialize, Default)]
struct AppConfig {
    work_minutes: u64,
    pause_minutes: u64,
    notification_mode: Option<NotificationMode>,
    hide_completed_tasks: Option<bool>,
}

impl AppConfig {
    fn get_config_path() -> PathBuf {
        // Use a local file for simplicity, or user data dir
        PathBuf::from("focus_timer_config.json")
    }

    fn load() -> Option<Self> {
        let path = Self::get_config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                return serde_json::from_str(&content).ok();
            }
        }
        None
    }

    fn save(&self) {
        let path = Self::get_config_path();
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, content);
        }
    }
}

impl TimerState {
    pub fn new(default_work_minutes: u64, default_pause_minutes: u64) -> Self {
        // Try to load config, otherwise use defaults
        let (work_minutes, pause_minutes, notif_mode, hide_completed) = if let Some(config) = AppConfig::load() {
            (
                config.work_minutes, 
                config.pause_minutes, 
                config.notification_mode.unwrap_or(NotificationMode::NotificationPersistent),
                config.hide_completed_tasks.unwrap_or(false)
            )
        } else {
            (default_work_minutes, default_pause_minutes, NotificationMode::NotificationPersistent, false)
        };

        let work_duration = Duration::from_secs(work_minutes * 60);
        let pause_duration = Duration::from_secs(pause_minutes * 60);
        
        Self {
            current_time: work_duration,
            work_duration,
            pause_duration,
            is_running: false,
            mode: TimerMode::Work,
            notification_mode: notif_mode,
            history: SessionHistory::load(),
            hide_completed_tasks: hide_completed,
        }
    }

    fn save_config(&self) {
        let config = AppConfig {
            work_minutes: self.work_duration.as_secs() / 60,
            pause_minutes: self.pause_duration.as_secs() / 60,
            notification_mode: Some(self.notification_mode),
            hide_completed_tasks: Some(self.hide_completed_tasks),
        };
        config.save();
    }

    /// Ticks the timer. Returns true if the timer just finished.
    pub fn tick(&mut self) -> bool {
        if self.is_running {
            if self.current_time.as_secs() > 0 {
                self.current_time = self.current_time.saturating_sub(Duration::from_secs(1));
                if self.current_time.as_secs() == 0 {
                    self.is_running = false;
                    return true;
                }
            } else {
                self.is_running = false;
            }
        }
        false
    }

    pub fn toggle(&mut self) {
        if self.current_time.as_secs() == 0 {
            // Reset if finished when toggled
            self.reset_current_mode();
            self.is_running = true;
        } else {
            self.is_running = !self.is_running;
        }
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
