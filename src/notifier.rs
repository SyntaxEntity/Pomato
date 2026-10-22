use crate::i18n::{fill, t};
use gtk::prelude::*;
use notify_rust::{Notification, Timeout};

pub fn phase_finished(phase: &str, next: &str, next_minutes: u32) {
    let _ = Notification::new()
        .summary(&fill(t("Pomato · {phase}结束"), &[("phase", phase)]))
        .body(&fill(
            t("下一阶段：{next} {next_minutes} 分钟"),
            &[("next", next), ("next_minutes", &next_minutes.to_string())],
        ))
        .timeout(Timeout::Milliseconds(6000))
        .show();
}

pub fn tray_unavailable_hint() {
    let _ = Notification::new()
        .summary(t("Pomato · 未检测到托盘扩展"))
        .body(t(
            "安装 gnome-shell-extension-appindicator 后可显示常驻托盘图标。应用仍会在后台运行。",
        ))
        .timeout(Timeout::Milliseconds(8000))
        .show();
}

pub fn beep() {
    if let Some(display) = gtk::gdk::Display::default() {
        display.beep();
    }
}
