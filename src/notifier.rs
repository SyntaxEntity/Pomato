use gtk::prelude::*;
use notify_rust::{Notification, Timeout};

pub fn phase_finished(phase: &str, next: &str, next_minutes: u32) {
    let _ = Notification::new()
        .summary(&format!("Pomato · {phase}结束"))
        .body(&format!("下一阶段：{next} {next_minutes} 分钟"))
        .timeout(Timeout::Milliseconds(6000))
        .show();
}

pub fn tray_unavailable_hint() {
    let _ = Notification::new()
        .summary("Pomato · 未检测到托盘扩展")
        .body("安装 gnome-shell-extension-appindicator 后可显示常驻托盘图标。应用仍会在后台运行。")
        .timeout(Timeout::Milliseconds(8000))
        .show();
}

pub fn beep() {
    if let Some(display) = gtk::gdk::Display::default() {
        display.beep();
    }
}
