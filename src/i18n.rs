//! 极简 i18n：中文原文即 msgid，按系统 locale 查表换成英文。
//!
//! 为什么不用 gettext：
//! 本项目只有中/英两种语言，且分发只有 RPM。走 gettext 需要 `.po`/`.mo`
//! 构建链路 + 把 `.mo` 安装到 `/usr/share/locale`，对当前规模收益不抵复杂度。
//! 这里用 glib 已有的 `get_user_language()` 检测语言，配一张静态表，
//! 不引入任何新依赖，也不改打包结构。
//!
//! 新增语言：在 `EN` 旁边加一张同结构的表，并在 `t()` 里加一个分支即可。
//! 文案里用 `{name}` 命名占位符，配合 [`fill`] 填充——用命名而非位置，
//! 是为了让译文能自由调整语序（例如「{tag} 还有 {n} 天」在英文里是
//! 「{n} days until {tag}」）。

use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Zh,
    En,
}

/// 按系统 locale 决定界面语言；无法判断时回落到中文。
pub fn lang() -> Lang {
    static LANG: OnceLock<Lang> = OnceLock::new();
    *LANG.get_or_init(|| {
        // 显式覆盖，便于测试：POMATO_LANG=en / zh
        if let Ok(v) = std::env::var("POMATO_LANG") {
            if v.to_ascii_lowercase().starts_with("en") {
                return Lang::En;
            }
            if v.to_ascii_lowercase().starts_with("zh") {
                return Lang::Zh;
            }
        }
        // 不依赖 setlocale：Rust 不会自动调用它，直接按 gettext 的优先级读环境变量。
        for key in ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"] {
            // 语义与 gettext 一致：只有真正的 en* locale 才切英文；
            // C / POSIX / 未设置时不做翻译，保持中文原文。
            if let Ok(v) = std::env::var(key)
                && !v.is_empty()
            {
                return if v.to_ascii_lowercase().starts_with("en") {
                    Lang::En
                } else {
                    Lang::Zh
                };
            }
        }
        Lang::Zh
    })
}

/// 翻译一条文案。未命中翻译时原样返回中文原文（保证永不会出现空白界面）。
pub fn t(zh: &'static str) -> &'static str {
    if lang() == Lang::Zh {
        return zh;
    }
    EN.iter()
        .find(|(k, _)| *k == zh)
        .map(|(_, v)| *v)
        .unwrap_or(zh)
}

/// 用命名占位符填充已翻译的模板：`fill(t("… {n} …"), &[("n", "3")])`。
pub fn fill(template: &str, args: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (k, v) in args {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

/// 英文文案表。键必须与源码里的中文原文**逐字一致**。
static EN: &[(&str, &str)] = &[
    // ---- 窗口 / 标题 ----
    ("番茄钟", "Pomodoro"),
    ("专注", "Focus"),
    ("短休", "Short break"),
    ("长休", "Long break"),
    ("阶段", "Phase"),
    ("计时", "Timer"),
    ("待开始 · ", "Not started · "),
    ("已暂停 · ", "Paused · "),
    (
        "第 {n} 个番茄 · 共 {interval} 个后长休",
        "Pomodoro {n} · long break every {interval}",
    ),
    // ---- 主界面按钮 ----
    ("开始", "Start"),
    ("暂停", "Pause"),
    ("重置", "Reset"),
    ("跳过", "Skip"),
    ("下一阶段：{next}", "Next: {next}"),
    (
        "下一阶段：{next} {next_minutes} 分钟",
        "Next: {next} for {next_minutes} min",
    ),
    ("{next} {mins} 分钟", "{next} for {mins} min"),
    // ---- 目标倒数日 ----
    ("目标倒数日", "Countdown"),
    ("未设置目标", "No target set"),
    ("未设置", "Not set"),
    ("已过去", "past"),
    ("日期", "Date"),
    ("清除日期", "Clear date"),
    ("日期无效", "Invalid date"),
    ("考研", "Final exams"),
    (
        "例如：考研 / 项目截止",
        "e.g. Final exams / Project deadline",
    ),
    ("{tag} 还有 {n} 天", "{n} days until {tag}"),
    ("{tag} 已过去 {n} 天", "{n} days since {tag}"),
    ("{tag} · 日期无效", "{tag} · invalid date"),
    ("今天是 {tag}", "Today is {tag}"),
    ("{y}年{m}月{d}日", "{m}/{d}/{y}"),
    (
        "主界面显示距目标日期的天数",
        "Show days until the target date on the main screen",
    ),
    ("名称", "Name"),
    // ---- 偏好设置 ----
    ("偏好设置", "Preferences"),
    ("时长（分钟）", "Duration (minutes)"),
    ("自动开始下一阶段", "Auto-start next phase"),
    ("长休间隔（番茄个数）", "Long break interval (pomodoros)"),
    ("提示音", "Sound"),
    ("休息一下，起来走动", "Take a break and stretch"),
    ("桌面通知", "Desktop notifications"),
    ("行为", "Behaviour"),
    ("最小化到托盘", "Minimise to tray"),
    (
        "关闭窗口时最小化到托盘",
        "Minimise to tray when window is closed",
    ),
    ("托盘", "Tray"),
    ("显示托盘图标", "Show tray icon"),
    ("常驻图标，建议保持开启", "Keep running; recommended on"),
    (
        "常驻图标；安装 AppIndicator 扩展后显示在 GNOME 顶栏",
        "Tray icon; needs the AppIndicator extension to appear in the GNOME top bar",
    ),
    ("计时继续在后台运行", "Timing continues in the background"),
    // ---- 全屏 / 背景 ----
    ("全屏 (F11)", "Fullscreen (F11)"),
    ("全屏背景", "Fullscreen background"),
    ("背景图片", "Background image"),
    ("背景模糊", "Background blur"),
    ("像素半径", "Blur radius (px)"),
    ("遮罩不透明度", "Overlay opacity"),
    ("遮罩颜色", "Overlay colour"),
    ("黑色 / 白色 / 自定义", "Black / White / Custom"),
    ("黑", "Black"),
    ("白", "White"),
    ("清除", "Clear"),
    ("选择…", "Choose…"),
    ("选择背景图片", "Choose background image"),
    ("图片", "Image"),
    (
        "未设置 · 全屏时使用 GTK 原生背景",
        "Not set · uses the GTK default background in fullscreen",
    ),
    (
        "仅全屏时生效；窗口模式保持 GTK 原生外观",
        "Fullscreen only; the window keeps the native GTK look",
    ),
    (
        "从日历选择目标日期",
        "Pick the target date from the calendar",
    ),
    ("点击选择目标日期", "Click to pick the target date"),
    ("0 = 透明，1 = 全黑", "0 = transparent, 1 = solid black"),
    // ---- 托盘菜单 ----
    ("显示 Pomato", "Show Pomato"),
    ("退出", "Quit"),
    // ---- 通知 ----
    ("Pomato · {phase}结束", "Pomato · {phase} finished"),
    (
        "Pomato · 未检测到托盘扩展",
        "Pomato · no tray extension detected",
    ),
    (
        "安装 gnome-shell-extension-appindicator 后可显示常驻托盘图标。应用仍会在后台运行。",
        "Install gnome-shell-extension-appindicator to keep a tray icon. The app keeps running in the background anyway.",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_replaces_named_placeholders() {
        assert_eq!(fill("a {x} b {y}", &[("x", "1"), ("y", "2")]), "a 1 b 2");
    }

    #[test]
    fn fill_leaves_unknown_placeholders() {
        assert_eq!(fill("{x} {z}", &[("x", "1")]), "1 {z}");
    }

    #[test]
    fn every_chinese_entry_has_ascii_key() {
        // 防呆：键里必须真的有中文，避免误把英文键写进来
        assert!(
            EN.iter()
                .all(|(k, _)| k.chars().any(|c| c as u32 >= 0x4e00))
        );
    }
}
