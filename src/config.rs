use std::path::PathBuf;

use gtk::gio::prelude::*;
use gtk::gio::{Settings, SettingsBackend, SettingsSchemaSource};

#[derive(Debug, Clone)]
pub struct Config {
    pub work_minutes: u32,
    pub short_break_minutes: u32,
    pub long_break_minutes: u32,
    pub long_break_interval: u32,
    pub auto_start_next: bool,
    pub sound_enabled: bool,
    pub notifications_enabled: bool,
    pub close_to_tray: bool,
    pub minimize_to_tray: bool,
    pub show_tray_icon: bool,
    pub goal_name: String,
    pub goal_date: String,
    pub fullscreen_bg_path: String,
    pub fullscreen_bg_dim: f64,
    pub fullscreen_bg_blur: f64,
    pub fullscreen_bg_color: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            long_break_interval: 4,
            auto_start_next: false,
            sound_enabled: true,
            notifications_enabled: true,
            close_to_tray: true,
            minimize_to_tray: true,
            show_tray_icon: true,
            goal_name: String::new(),
            goal_date: String::new(),
            fullscreen_bg_path: String::new(),
            fullscreen_bg_dim: 0.45,
            fullscreen_bg_blur: 12.0,
            fullscreen_bg_color: "black".to_string(),
        }
    }
}

impl Config {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            work_minutes: settings.uint("work-minutes"),
            short_break_minutes: settings.uint("short-break-minutes"),
            long_break_minutes: settings.uint("long-break-minutes"),
            long_break_interval: settings.uint("long-break-interval"),
            auto_start_next: settings.boolean("auto-start-next"),
            sound_enabled: settings.boolean("sound-enabled"),
            notifications_enabled: settings.boolean("notifications-enabled"),
            close_to_tray: settings.boolean("close-to-tray"),
            minimize_to_tray: settings.boolean("minimize-to-tray"),
            show_tray_icon: settings.boolean("show-tray-icon"),
            goal_name: settings.string("goal-name").to_string(),
            goal_date: settings.string("goal-date").to_string(),
            fullscreen_bg_path: settings.string("fullscreen-bg-path").to_string(),
            fullscreen_bg_dim: settings.double("fullscreen-bg-dim"),
            fullscreen_bg_blur: settings.double("fullscreen-bg-blur"),
            fullscreen_bg_color: settings.string("fullscreen-bg-color").to_string(),
        }
    }
}

/// Parse `YYYY-MM-DD` into (year, month, day).
pub fn parse_date(s: &str) -> Option<(i32, u32, u32)> {
    let s = s.trim();
    let mut parts = s.split('-');
    let y: i32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    let d: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some((y, m, d))
}

/// Days since 1970-01-01 (Howard Hinnant civil calendar).
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = ((153 * (if m > 2 { m - 3 } else { m + 9 }) as i64 + 2) / 5) + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Days from today until `goal_date` (`YYYY-MM-DD`). Negative = already past.
pub fn days_until(goal_date: &str) -> Option<i64> {
    let (gy, gm, gd) = parse_date(goal_date)?;
    let now = gtk::glib::DateTime::now_local().ok()?;
    let today = days_from_civil(
        now.year() as i64,
        now.month() as u32,
        now.day_of_month() as u32,
    );
    Some(days_from_civil(gy as i64, gm, gd) - today)
}

/// Human-readable goal countdown line for the main UI.
pub fn goal_label(cfg: &Config) -> String {
    let date = cfg.goal_date.trim();
    if date.is_empty() {
        return "未设置目标".to_string();
    }
    let name = cfg.goal_name.trim();
    let tag = if name.is_empty() {
        date.to_string()
    } else {
        name.to_string()
    };
    match days_until(date) {
        Some(0) => format!("今天是 {tag}"),
        Some(n) if n > 0 => format!("{tag} 还有 {n} 天"),
        Some(n) => format!("{tag} 已过去 {} 天", -n),
        None => format!("{tag} · 日期无效"),
    }
}

fn find_settings() -> Option<Settings> {
    if let Some(schema) =
        SettingsSchemaSource::default().and_then(|s| s.lookup("io.github.pomato.Pomato", false))
    {
        return Some(Settings::new_full(&schema, None::<&SettingsBackend>, None));
    }

    let dev = PathBuf::from(env!("POMATO_SCHEMA_DIR"));
    let source = SettingsSchemaSource::from_directory(&dev, None, false).ok()?;
    let schema = source.lookup("io.github.pomato.Pomato", false)?;
    Some(Settings::new_full(&schema, None::<&SettingsBackend>, None))
}

pub fn load() -> (Option<Settings>, Config) {
    match find_settings() {
        Some(settings) => {
            let config = Config::from_settings(&settings);
            (Some(settings), config)
        }
        None => {
            eprintln!("GSettings schema not found; using defaults");
            (None, Config::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_date() {
        assert_eq!(parse_date("2026-06-01"), Some((2026, 6, 1)));
    }

    #[test]
    fn parse_rejects_bad_dates() {
        assert!(parse_date("").is_none());
        assert!(parse_date("not-a-date").is_none());
        assert!(parse_date("2026-13-01").is_none());
        assert!(parse_date("2026-01-99").is_none());
    }

    #[test]
    fn days_since_epoch() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(1970, 1, 2), 1);
        assert_eq!(
            days_from_civil(2026, 1, 1),
            days_from_civil(2025, 1, 1) + 365
        );
    }

    #[test]
    fn goal_label_states() {
        let mut cfg = Config::default();
        assert_eq!(goal_label(&cfg), "未设置目标");
        cfg.goal_name = "考研".into();
        cfg.goal_date = "2020-01-01".into();
        assert!(goal_label(&cfg).contains("已过去"));
        cfg.goal_date = "2020-13-01".into();
        assert!(goal_label(&cfg).contains("日期无效"));
    }
}
