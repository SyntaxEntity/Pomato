use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub fn title(self) -> &'static str {
        match self {
            Phase::Work => "专注",
            Phase::ShortBreak => "短休",
            Phase::LongBreak => "长休",
        }
    }

    pub fn next_after_work(completed_work: u32, long_break_interval: u32) -> Phase {
        if completed_work > 0 && completed_work % long_break_interval == 0 {
            Phase::LongBreak
        } else {
            Phase::ShortBreak
        }
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.title())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerStatus {
    Idle,
    Running,
    Paused,
}

#[derive(Debug, Clone)]
pub struct Timer {
    pub phase: Phase,
    pub status: TimerStatus,
    pub remaining_secs: u32,
    pub completed_work: u32,
    pub config: crate::config::Config,
}

impl Timer {
    pub fn new(config: crate::config::Config) -> Self {
        let remaining_secs = config.work_minutes * 60;
        Self {
            phase: Phase::Work,
            status: TimerStatus::Idle,
            remaining_secs,
            completed_work: 0,
            config,
        }
    }

    pub fn apply_config(&mut self, config: crate::config::Config) {
        self.config = config;
        if self.status == TimerStatus::Idle {
            self.remaining_secs = self.phase_duration_secs(self.phase);
        }
    }

    pub fn phase_duration_secs(&self, phase: Phase) -> u32 {
        match phase {
            Phase::Work => self.config.work_minutes * 60,
            Phase::ShortBreak => self.config.short_break_minutes * 60,
            Phase::LongBreak => self.config.long_break_minutes * 60,
        }
    }

    pub fn start(&mut self) {
        if self.status != TimerStatus::Running {
            self.status = TimerStatus::Running;
        }
    }

    pub fn pause(&mut self) {
        if self.status == TimerStatus::Running {
            self.status = TimerStatus::Paused;
        }
    }

    pub fn toggle(&mut self) {
        match self.status {
            TimerStatus::Running => self.pause(),
            TimerStatus::Idle | TimerStatus::Paused => self.start(),
        }
    }

    pub fn reset(&mut self) {
        self.status = TimerStatus::Idle;
        self.remaining_secs = self.phase_duration_secs(self.phase);
    }

    pub fn skip(&mut self) {
        self.advance_phase(false);
    }

    /// Returns true when a phase just completed (for notification side-effects).
    pub fn tick(&mut self) -> bool {
        if self.status != TimerStatus::Running {
            return false;
        }
        if self.remaining_secs == 0 {
            return false;
        }
        self.remaining_secs -= 1;
        if self.remaining_secs == 0 {
            self.advance_phase(true);
            return true;
        }
        false
    }

    fn advance_phase(&mut self, completed: bool) {
        match self.phase {
            Phase::Work => {
                if completed {
                    self.completed_work += 1;
                }
                self.phase = Phase::next_after_work(self.completed_work, self.config.long_break_interval);
            }
            Phase::ShortBreak | Phase::LongBreak => {
                self.phase = Phase::Work;
            }
        }
        self.remaining_secs = self.phase_duration_secs(self.phase);
        if self.config.auto_start_next {
            self.status = TimerStatus::Running;
        } else {
            self.status = TimerStatus::Idle;
        }
    }

    pub fn progress(&self) -> f64 {
        let total = self.phase_duration_secs(self.phase).max(1) as f64;
        let done = total - self.remaining_secs as f64;
        (done / total).clamp(0.0, 1.0)
    }

    pub fn display_time(&self) -> String {
        let m = self.remaining_secs / 60;
        let s = self.remaining_secs % 60;
        format!("{m:02}:{s:02}")
    }

    pub fn session_label(&self) -> String {
        let interval = self.config.long_break_interval.max(1);
        match self.phase {
            Phase::Work => {
                let n = (self.completed_work % interval) + 1;
                format!("第 {n} 个番茄 · 共 {interval} 个后长休")
            }
            _ => "休息一下，起来走动".to_string(),
        }
    }

    pub fn tray_title(&self) -> String {
        let state = match self.status {
            TimerStatus::Running => "",
            TimerStatus::Paused => "已暂停 · ",
            TimerStatus::Idle => "待开始 · ",
        };
        format!("{state}{} · {}", self.phase.title(), self.display_time())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn short_config() -> Config {
        Config {
            work_minutes: 1,
            short_break_minutes: 1,
            long_break_minutes: 2,
            long_break_interval: 2,
            auto_start_next: true,
            ..Config::default()
        }
    }

    #[test]
    fn work_then_short_break() {
        let mut t = Timer::new(short_config());
        t.start();
        for _ in 0..60 {
            assert!(!t.tick() || t.phase == Phase::ShortBreak || t.phase == Phase::LongBreak);
        }
        assert_eq!(t.phase, Phase::ShortBreak);
        assert_eq!(t.completed_work, 1);
    }

    #[test]
    fn long_break_after_interval() {
        let mut t = Timer::new(short_config());
        t.completed_work = 1;
        t.phase = Phase::Work;
        t.remaining_secs = 1;
        t.start();
        assert!(t.tick());
        assert_eq!(t.phase, Phase::LongBreak);
    }

    #[test]
    fn reset_restores_phase_duration() {
        let mut t = Timer::new(short_config());
        t.start();
        t.tick();
        t.reset();
        assert_eq!(t.remaining_secs, 60);
        assert_eq!(t.status, TimerStatus::Idle);
    }

    #[test]
    fn skip_advances_without_counting_work() {
        let mut t = Timer::new(short_config());
        t.skip();
        assert_eq!(t.phase, Phase::ShortBreak);
        assert_eq!(t.completed_work, 0);
    }
}
