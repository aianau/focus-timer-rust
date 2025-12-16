use std::fmt;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimerMode {
    Work,
    Pause,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimerState {
    pub current_time: Duration,
    pub work_duration: Duration,
    pub pause_duration: Duration,
    pub is_running: bool,
    pub mode: TimerMode,
    pub notification_mode: NotificationMode,
}

impl TimerState {
    pub fn new(work_minutes: u64, pause_minutes: u64) -> Self {
        let work_duration = Duration::from_secs(work_minutes * 60);
        let pause_duration = Duration::from_secs(pause_minutes * 60);

        Self {
            current_time: work_duration,
            work_duration,
            pause_duration,
            is_running: false,
            mode: TimerMode::Work,
            notification_mode: NotificationMode::NotificationPersistent,
        }
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
    }

    pub fn set_pause_duration(&mut self, minutes: u64) {
        self.pause_duration = Duration::from_secs(minutes * 60);
        if self.mode == TimerMode::Pause && !self.is_running {
            self.current_time = self.pause_duration;
        }
    }

    pub fn set_notification_mode(&mut self, mode: NotificationMode) {
        self.notification_mode = mode;
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
