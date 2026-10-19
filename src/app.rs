use adw::prelude::*;
use gtk::cairo;
use gtk::gio::Settings;
use gtk::glib;
use gtk::glib::ControlFlow;
use relm4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use crate::config::{Config, goal_label, load};
use crate::notifier;
use crate::timer::{Phase, Timer, TimerStatus};
use crate::tray::{self, TrayEvent, TrayHandle};

#[derive(Debug)]
pub enum Msg {
    Tick,
    Toggle,
    Reset,
    Skip,
    OpenPreferences,
    ConfigChanged(Config),
    Tray(TrayEvent),
    HideWindow,
    ToggleFullscreen,
    ExitFullscreen,
    /// The window's real `fullscreened` property changed — driven by
    /// `notify::fullscreened`, so it also fires when the compositor (Super,
    /// drag, hot corner) changes fullscreen behind our back.
    FullscreenChanged(bool),
    Quit,
}

pub struct App {
    timer: Timer,
    settings: Option<Settings>,
    tray: Option<TrayHandle>,
    window: Option<adw::ApplicationWindow>,
    ring: Option<gtk::DrawingArea>,
    progress: Rc<Cell<f64>>,
    bg_picture: Option<gtk::Picture>,
    bg_scrim: Option<gtk::Box>,
    bg_css: Option<gtk::CssProvider>,
}

#[relm4::component(pub)]
impl SimpleComponent for App {
    type Init = ();
    type Input = Msg;
    type Output = ();

    view! {
        adw::ApplicationWindow {
            set_title: Some("Pomato"),
            // 默认尺寸取在内容最小尺寸之上（实测：内容 min 578×699，
            // 加 HeaderBar 后窗口 min 578×745），留一点余量。
            // 窗口拖不到比 min 更小 —— 这条下限由内容 measure 自然传播，
            // 不依赖 set_size_request（见 set_child 处的说明）。
            set_default_width: 620,
            set_default_height: 780,

            connect_close_request[sender] => move |_window| {
                sender.input(Msg::HideWindow);
                glib::Propagation::Stop
            },

            adw::ToolbarView {
                #[name = "header"]
                add_top_bar = &adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Pomato",
                        set_subtitle: "番茄钟",
                    },

                    pack_end = &gtk::Button {
                        set_icon_name: "emblem-system-symbolic",
                        set_tooltip_text: Some("偏好设置"),
                        connect_clicked[sender] => move |_| {
                            sender.input(Msg::OpenPreferences)
                        },
                    },

                    pack_end = &gtk::Button {
                        set_icon_name: "view-fullscreen-symbolic",
                        set_tooltip_text: Some("全屏 (F11)"),
                        connect_clicked[sender] => move |_| {
                            sender.input(Msg::ToggleFullscreen)
                        },
                    },
                },

                #[wrap(Some)]
                set_content = &gtk::Overlay {
                    #[name = "bg_picture"]
                    gtk::Picture {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_content_fit: gtk::ContentFit::Cover,
                        set_visible: false,
                        add_css_class: "bg-picture",
                    },

                    #[name = "bg_scrim"]
                    add_overlay = &gtk::Box {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_visible: false,
                        add_css_class: "bg-scrim",
                    },

                    // ⚠️ 内容必须是 Overlay 的 **child**（set_child），不能是 add_overlay。
                    // gtk::Overlay 只测量主 child，overlay 子件不参与尺寸计算：
                    // 若内容也挂在 add_overlay 上，Overlay 的 min 会是 0x0，
                    // 最小尺寸就永远传播不到窗口（2026-10-20 实测踩到）。
                    // 两张背景图（bg_picture / bg_scrim）继续做 overlay，盖在内容之上。
                    #[name = "content"]
                    #[wrap(Some)]
                    set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    set_vexpand: true,
                    add_css_class: "app-content",
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Fill,
                    gtk::CenterBox {
                        set_vexpand: true,
                        set_hexpand: true,
                        set_margin_top: 24,
                        set_margin_bottom: 24,
                        set_margin_start: 24,
                        set_margin_end: 24,

                        #[wrap(Some)]
                        set_center_widget = &gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            set_spacing: 24,

                            gtk::Label {
                                set_halign: gtk::Align::Center,
                                add_css_class: "phase-title",
                                #[watch]
                                set_label: model.timer.phase.title(),
                            },

                            gtk::Overlay {
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,

                                #[name = "ring"]
                                gtk::DrawingArea {
                                    set_content_width: 340,
                                    set_content_height: 340,
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::Center,
                                },

                                add_overlay = &gtk::Label {
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::Center,
                                    add_css_class: "timer-display",
                                    #[watch]
                                    set_label: &model.timer.display_time(),
                                },
                            },

                            gtk::Label {
                                set_halign: gtk::Align::Center,
                                add_css_class: "dim-label",
                                add_css_class: "session-label",
                                #[watch]
                                set_label: &model.timer.session_label(),
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_halign: gtk::Align::Center,
                                set_spacing: 16,
                                set_margin_top: 8,

                                gtk::Button {
                                    add_css_class: "suggested-action",
                                    add_css_class: "pill",
                                    add_css_class: "action-btn",
                                    #[watch]
                                    set_label: match model.timer.status {
                                        TimerStatus::Running => "暂停",
                                        _ => "开始",
                                    },
                                    connect_clicked[sender] => move |_| sender.input(Msg::Toggle),
                                },

                                gtk::Button {
                                    set_label: "重置",
                                    add_css_class: "pill",
                                    add_css_class: "action-btn",
                                    connect_clicked[sender] => move |_| sender.input(Msg::Reset),
                                },

                                gtk::Button {
                                    set_label: "跳过",
                                    add_css_class: "pill",
                                    add_css_class: "action-btn",
                                    connect_clicked[sender] => move |_| sender.input(Msg::Skip),
                                },
                            },

                            gtk::Label {
                                set_halign: gtk::Align::Center,
                                add_css_class: "goal-countdown",
                                #[watch]
                                set_label: &goal_label(&model.timer.config),
                            },

                            gtk::Label {
                                set_halign: gtk::Align::Center,
                                add_css_class: "caption",
                                add_css_class: "next-hint",
                                #[watch]
                                set_label: &format!("下一阶段：{}", next_phase_preview(&model.timer)),
                            },
                        },
                    },
                    },
                },
            },
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (settings, config) = load();
        let timer = Timer::new(config);

        let mut model = App {
            timer,
            settings,
            tray: None,
            window: Some(root.clone()),
            ring: None,
            progress: Rc::new(Cell::new(0.0)),
            bg_picture: None,
            bg_scrim: None,
            bg_css: None,
        };

        let widgets = view_output!();

        // 窗口首次映射后强制整窗重绘一次。
        // 背景：窗口刚打开时最右边缘会出现 1-2px 暗线（截图实测：内容 #fafafa →
        // #979797 → (1,1,1) 的硬边），拖动窗口后消失 —— 典型的首帧未画满。
        // 这里在 map 之后再排一次 redraw，确保整块 surface 被完整绘制。
        {
            let win = root.clone();
            root.connect_map(move |_| {
                let win = win.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(50), move || {
                    win.queue_draw();
                });
            });
        }

        // Circular progress ring — redrawn each tick from the shared cell.
        {
            let progress = model.progress.clone();
            widgets.ring.set_draw_func(move |_, cr, w, h| {
                draw_progress_ring(cr, w as f64, h as f64, progress.get());
            });
        }
        model.ring = Some(widgets.ring.clone());
        model.bg_picture = Some(widgets.bg_picture.clone());
        model.bg_scrim = Some(widgets.bg_scrim.clone());
        {
            let provider = gtk::CssProvider::new();
            if let Some(display) = gtk::gdk::Display::default() {
                gtk::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk::STYLE_PROVIDER_PRIORITY_USER,
                );
            }
            model.bg_css = Some(provider);
        }

        // Keyboard: F11 toggles fullscreen, Escape leaves it.
        {
            let sender = sender.clone();
            let key = gtk::EventControllerKey::new();
            key.connect_key_pressed(move |_, keyval, _code, _mods| {
                if keyval == gtk::gdk::Key::F11 {
                    sender.input(Msg::ToggleFullscreen);
                    return glib::Propagation::Stop;
                }
                if keyval == gtk::gdk::Key::Escape {
                    sender.input(Msg::ExitFullscreen);
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });
            root.add_controller(key);
        }

        // Fullscreen truth comes from the window itself, never from a mirror
        // flag. `notify::fullscreened` also fires when the compositor leaves
        // fullscreen without going through our Messages (Super, drag, hot
        // corner), so the fullscreen-only background can never go stale.
        {
            let sender = sender.clone();
            root.connect_fullscreened_notify(move |window| {
                sender.input(Msg::FullscreenChanged(window.is_fullscreen()));
            });
        }

        {
            let sender = sender.clone();
            glib::timeout_add_seconds_local(1, move || {
                sender.input(Msg::Tick);
                ControlFlow::Continue
            });
        }

        {
            let sender = sender.clone();
            if let Some(settings) = model.settings.clone() {
                settings.connect_changed(None, move |settings, _key| {
                    sender.input(Msg::ConfigChanged(Config::from_settings(settings)));
                });
            }
        }

        match tray::spawn() {
            Ok((handle, events)) => {
                handle.set_title(model.timer.tray_title());
                handle.set_running(model.timer.status == TimerStatus::Running);
                {
                    let sender = sender.clone();
                    relm4::spawn(async move {
                        while let Ok(event) = events.recv_async().await {
                            sender.input(Msg::Tray(event));
                        }
                    });
                }
                model.tray = Some(handle);
            }
            Err(err) => {
                eprintln!("tray unavailable: {err}");
                if model.timer.config.show_tray_icon {
                    notifier::tray_unavailable_hint();
                }
            }
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            Msg::Tick => {
                let finished = self.timer.tick();
                if finished {
                    self.on_phase_finished();
                }
                self.progress.set(self.timer.progress());
                if let Some(ring) = &self.ring {
                    ring.queue_draw();
                }
                self.sync_tray();
            }
            Msg::Toggle => {
                self.timer.toggle();
                self.progress.set(self.timer.progress());
                if let Some(ring) = &self.ring {
                    ring.queue_draw();
                }
                self.sync_tray();
            }
            Msg::Reset => {
                self.timer.reset();
                self.progress.set(self.timer.progress());
                if let Some(ring) = &self.ring {
                    ring.queue_draw();
                }
                self.sync_tray();
            }
            Msg::Skip => {
                self.timer.skip();
                self.progress.set(self.timer.progress());
                if let Some(ring) = &self.ring {
                    ring.queue_draw();
                }
                self.sync_tray();
            }
            Msg::OpenPreferences => self.open_preferences(),
            Msg::ConfigChanged(config) => {
                self.timer.apply_config(config);
                self.progress.set(self.timer.progress());
                if let Some(ring) = &self.ring {
                    ring.queue_draw();
                }
                // Re-apply against the window's real state, not a mirror flag.
                let fullscreened = self
                    .window
                    .as_ref()
                    .is_some_and(|window| window.is_fullscreen());
                self.apply_fullscreen_bg(fullscreened);
                self.sync_tray();
            }
            Msg::Tray(event) => match event {
                TrayEvent::Show => self.present_window(),
                TrayEvent::Toggle => {
                    self.timer.toggle();
                    self.sync_tray();
                }
                TrayEvent::Reset => {
                    self.timer.reset();
                    self.sync_tray();
                }
                TrayEvent::Skip => {
                    self.timer.skip();
                    self.sync_tray();
                }
                TrayEvent::Quit => sender.input(Msg::Quit),
            },
            Msg::HideWindow => {
                let prefer_tray =
                    self.timer.config.close_to_tray || self.timer.config.minimize_to_tray;
                if prefer_tray && self.timer.config.show_tray_icon {
                    if let Some(window) = &self.window {
                        window.set_visible(false);
                    }
                } else {
                    sender.input(Msg::Quit);
                }
            }
            Msg::ToggleFullscreen => {
                // Ask the window what it currently is, instead of trusting a
                // mirrored flag that compositor-side changes would desync.
                let Some(window) = &self.window else {
                    return;
                };
                if window.is_fullscreen() {
                    window.unfullscreen();
                } else {
                    window.fullscreen();
                }
                // No apply_fullscreen_bg() here: the property notification
                // drives Msg::FullscreenChanged once GTK really switches.
            }
            Msg::ExitFullscreen => {
                if let Some(window) = &self.window {
                    // Unconditional and idempotent: no-op when not fullscreen.
                    window.unfullscreen();
                }
            }
            Msg::FullscreenChanged(fullscreened) => {
                self.apply_fullscreen_bg(fullscreened);
            }
            Msg::Quit => {
                if self.settings.is_some() {
                    Settings::sync();
                }
                if let Some(window) = &self.window {
                    window.destroy();
                }
                relm4::main_application().quit();
            }
        }
    }
}

impl App {
    fn sync_tray(&mut self) {
        let Some(tray) = &self.tray else {
            return;
        };
        tray.set_title(self.timer.tray_title());
        tray.set_running(self.timer.status == TimerStatus::Running);
        let icon = match self.timer.phase {
            Phase::Work => "alarm-symbolic",
            Phase::ShortBreak => "weather-clear-symbolic",
            Phase::LongBreak => "emblem-favorite-symbolic",
        };
        tray.set_icon_name(icon);
    }

    fn present_window(&self) {
        if let Some(window) = &self.window {
            window.set_visible(true);
            window.present();
        }
    }

    /// Fullscreen-only custom background: image + dark scrim + blur.
    /// Windowed mode stays native GTK (picture/scrim hidden).
    ///
    /// `fullscreened` is the window's real state, threaded in by the caller so
    /// this decision can never disagree with the compositor.
    fn apply_fullscreen_bg(&self, fullscreened: bool) {
        let Some(picture) = &self.bg_picture else {
            return;
        };
        let Some(scrim) = &self.bg_scrim else {
            return;
        };
        if !fullscreened {
            picture.set_visible(false);
            scrim.set_visible(false);
            return;
        }
        let cfg = &self.timer.config;
        let path = cfg.fullscreen_bg_path.trim();
        if path.is_empty() {
            picture.set_visible(false);
            scrim.set_visible(false);
            return;
        }
        picture.set_filename(Some(path));
        picture.set_visible(true);
        scrim.set_visible(true);
        scrim.set_opacity(cfg.fullscreen_bg_dim.clamp(0.0, 1.0));
        let blur = cfg.fullscreen_bg_blur.clamp(0.0, 64.0);
        let color = cfg.fullscreen_bg_color.trim();
        let color = if color.is_empty() { "black" } else { color };
        if let Some(provider) = &self.bg_css {
            provider.load_from_string(&format!(
                "picture.bg-picture {{ filter: blur({blur:.1}px); }}\nbox.bg-scrim {{ background-color: {color}; }}"
            ));
        }
    }

    fn on_phase_finished(&self) {
        if self.timer.config.sound_enabled {
            notifier::beep();
        }
        if self.timer.config.notifications_enabled {
            let next = self.timer.phase.title();
            let mins = self.timer.phase_duration_secs(self.timer.phase) / 60;
            notifier::phase_finished("阶段", next, mins);
        }
    }

    fn open_preferences(&self) {
        let Some(parent) = self.window.as_ref() else {
            return;
        };
        let Some(settings) = self.settings.clone() else {
            return;
        };

        let dialog = adw::PreferencesDialog::new();
        dialog.set_title("偏好设置");

        let page = adw::PreferencesPage::new();
        page.set_title("计时");
        page.set_icon_name(Some("preferences-system-time-symbolic"));

        let duration_group = adw::PreferencesGroup::new();
        duration_group.set_title("时长（分钟）");
        duration_group.add(&spin_row(&settings, "work-minutes", "专注", 1, 90));
        duration_group.add(&spin_row(&settings, "short-break-minutes", "短休", 1, 90));
        duration_group.add(&spin_row(&settings, "long-break-minutes", "长休", 1, 90));
        duration_group.add(&spin_row(
            &settings,
            "long-break-interval",
            "长休间隔（番茄个数）",
            1,
            12,
        ));

        let behavior_group = adw::PreferencesGroup::new();
        behavior_group.set_title("行为");
        behavior_group.add(&switch_row(
            &settings,
            "auto-start-next",
            "自动开始下一阶段",
            None,
        ));
        behavior_group.add(&switch_row(&settings, "sound-enabled", "提示音", None));
        behavior_group.add(&switch_row(
            &settings,
            "notifications-enabled",
            "桌面通知",
            None,
        ));

        let tray_group = adw::PreferencesGroup::new();
        tray_group.set_title("托盘");
        tray_group.set_description(Some("常驻图标；安装 AppIndicator 扩展后显示在 GNOME 顶栏"));
        tray_group.add(&switch_row(
            &settings,
            "close-to-tray",
            "关闭窗口时最小化到托盘",
            Some("计时继续在后台运行"),
        ));
        tray_group.add(&switch_row(
            &settings,
            "minimize-to-tray",
            "最小化到托盘",
            None,
        ));
        tray_group.add(&switch_row(
            &settings,
            "show-tray-icon",
            "显示托盘图标",
            Some("常驻图标，建议保持开启"),
        ));

        let goal_group = adw::PreferencesGroup::new();
        goal_group.set_title("目标倒数日");
        goal_group.set_description(Some("主界面显示距目标日期的天数"));
        goal_group.add(&entry_row(
            &settings,
            "goal-name",
            "名称",
            "例如：考研 / 项目截止",
        ));
        goal_group.add(&goal_date_row(&settings));

        let bg_group = adw::PreferencesGroup::new();
        bg_group.set_title("全屏背景");
        bg_group.set_description(Some("仅全屏时生效；窗口模式保持 GTK 原生外观"));
        bg_group.add(&bg_image_row(&settings, parent));
        bg_group.add(&bg_color_row(&settings));
        bg_group.add(&double_spin_row(
            &settings,
            "fullscreen-bg-dim",
            "遮罩不透明度",
            0.0,
            1.0,
            0.05,
            Some("0 = 透明，1 = 全黑"),
        ));
        bg_group.add(&double_spin_row(
            &settings,
            "fullscreen-bg-blur",
            "背景模糊",
            0.0,
            64.0,
            1.0,
            Some("像素半径"),
        ));

        page.add(&goal_group);
        page.add(&bg_group);
        page.add(&duration_group);
        page.add(&behavior_group);
        page.add(&tray_group);
        dialog.add(&page);
        dialog.present(Some(parent));
    }
}

/// Large circular progress ring for fullscreen / glanceable use.
fn draw_progress_ring(cr: &cairo::Context, w: f64, h: f64, progress: f64) {
    let (w, h) = (w as f64, h as f64);
    let cx = w / 2.0;
    let cy = h / 2.0;
    let line_width = 20.0;
    let radius = (w.min(h) / 2.0) - line_width;

    // Track
    cr.set_line_width(line_width);
    cr.set_line_cap(cairo::LineCap::Round);
    cr.set_source_rgba(0.5, 0.5, 0.5, 0.18);
    let _ = cr.arc(cx, cy, radius, 0.0, std::f64::consts::TAU);
    let _ = cr.stroke();

    // Progress arc — tomato red, from top, clockwise.
    if progress > 0.0 {
        let start = -std::f64::consts::FRAC_PI_2;
        let end = start + progress.clamp(0.0, 1.0) * std::f64::consts::TAU;
        cr.set_source_rgb(0.88, 0.18, 0.15);
        let _ = cr.arc(cx, cy, radius, start, end);
        let _ = cr.stroke();
    }
}

fn spin_row(settings: &Settings, key: &str, title: &str, min: u32, max: u32) -> adw::SpinRow {
    let row = adw::SpinRow::with_range(min as f64, max as f64, 1.0);
    row.set_title(title);
    row.set_value(settings.uint(key) as f64);
    let settings = settings.clone();
    let key = key.to_string();
    row.connect_changed(move |row| {
        let _ = settings.set_uint(&key, row.value() as u32);
    });
    row
}

fn double_spin_row(
    settings: &Settings,
    key: &str,
    title: &str,
    min: f64,
    max: f64,
    step: f64,
    subtitle: Option<&str>,
) -> adw::SpinRow {
    let row = adw::SpinRow::with_range(min, max, step);
    row.set_title(title);
    if let Some(sub) = subtitle {
        row.set_subtitle(sub);
    }
    row.set_value(settings.double(key));
    let settings = settings.clone();
    let key = key.to_string();
    row.connect_changed(move |row| {
        let _ = settings.set_double(&key, row.value());
    });
    row
}

/// Mask color: black / white presets + custom ColorDialogButton.
fn bg_color_row(settings: &Settings) -> adw::ActionRow {
    let row = adw::ActionRow::new();
    row.set_title("遮罩颜色");
    row.set_subtitle("黑色 / 白色 / 自定义");

    let dialog = gtk::ColorDialog::new();
    dialog.set_title("遮罩颜色");
    dialog.set_modal(true);
    dialog.set_with_alpha(false);

    let swatch = gtk::ColorDialogButton::new(Some(dialog.clone()));
    if let Ok(rgba) = gtk::gdk::RGBA::parse(settings.string("fullscreen-bg-color").as_str()) {
        swatch.set_rgba(&rgba);
    }

    let black = gtk::Button::with_label("黑");
    black.add_css_class("flat");
    let white = gtk::Button::with_label("白");
    white.add_css_class("flat");

    {
        let settings = settings.clone();
        let swatch = swatch.clone();
        black.connect_clicked(move |_| {
            if let Ok(rgba) = gtk::gdk::RGBA::parse("black") {
                swatch.set_rgba(&rgba);
                let _ = settings.set_string("fullscreen-bg-color", "black");
            }
        });
    }
    {
        let settings = settings.clone();
        let swatch = swatch.clone();
        white.connect_clicked(move |_| {
            if let Ok(rgba) = gtk::gdk::RGBA::parse("white") {
                swatch.set_rgba(&rgba);
                let _ = settings.set_string("fullscreen-bg-color", "white");
            }
        });
    }
    {
        let settings = settings.clone();
        swatch.connect_rgba_notify(move |btn| {
            let text = btn.rgba().to_string();
            let _ = settings.set_string("fullscreen-bg-color", &text);
        });
    }

    let box_ = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    box_.append(&black);
    box_.append(&white);
    box_.append(&swatch);
    row.add_suffix(&box_);
    row
}

/// Background image picker for fullscreen mode.
fn bg_image_row(settings: &Settings, parent: &adw::ApplicationWindow) -> adw::ActionRow {
    let row = adw::ActionRow::new();
    row.set_title("背景图片");

    let current = settings.string("fullscreen-bg-path").to_string();
    if current.is_empty() {
        row.set_subtitle("未设置 · 全屏时使用 GTK 原生背景");
    } else {
        let name = std::path::Path::new(&current)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| current.clone());
        row.set_subtitle(&name);
    }

    let clear = gtk::Button::with_label("清除");
    clear.add_css_class("flat");
    clear.set_sensitive(!current.is_empty());
    {
        let settings = settings.clone();
        let row = row.clone();
        clear.connect_clicked(move |_| {
            let _ = settings.set_string("fullscreen-bg-path", "");
            row.set_subtitle("未设置 · 全屏时使用 GTK 原生背景");
        });
    }

    let pick = gtk::Button::with_label("选择…");
    pick.add_css_class("suggested-action");
    {
        let settings = settings.clone();
        let row = row.clone();
        let clear = clear.clone();
        let parent = parent.clone();
        pick.connect_clicked(move |_| {
            let dialog = gtk::FileDialog::new();
            dialog.set_title("选择背景图片");
            let filter = gtk::FileFilter::new();
            filter.set_name(Some("图片"));
            filter.add_mime_type("image/*");
            let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
            filters.append(&filter);
            dialog.set_filters(Some(&filters));
            if !settings.string("fullscreen-bg-path").is_empty() {
                dialog.set_initial_file(Some(&gtk::gio::File::for_path(
                    settings.string("fullscreen-bg-path").as_str(),
                )));
            }
            let settings = settings.clone();
            let row = row.clone();
            let clear = clear.clone();
            dialog.open(Some(&parent), gtk::gio::Cancellable::NONE, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        let path = path.to_string_lossy().into_owned();
                        let _ = settings.set_string("fullscreen-bg-path", &path);
                        let name = std::path::Path::new(&path)
                            .file_name()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or(path);
                        row.set_subtitle(&name);
                        clear.set_sensitive(true);
                    }
                }
            });
        });
    }

    let box_ = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    box_.append(&clear);
    box_.append(&pick);
    row.add_suffix(&box_);
    row
}

fn switch_row(
    settings: &Settings,
    key: &str,
    title: &str,
    subtitle: Option<&str>,
) -> adw::SwitchRow {
    let row = adw::SwitchRow::new();
    row.set_title(title);
    if let Some(sub) = subtitle {
        row.set_subtitle(sub);
    }
    row.set_active(settings.boolean(key));
    let settings = settings.clone();
    let key = key.to_string();
    row.connect_active_notify(move |row| {
        let _ = settings.set_boolean(&key, row.is_active());
    });
    row
}

/// Goal date picker: popover calendar writing `YYYY-MM-DD` into GSettings.
fn goal_date_row(settings: &Settings) -> adw::ActionRow {
    let row = adw::ActionRow::new();
    row.set_title("日期");
    row.set_subtitle("点击选择目标日期");

    let calendar = gtk::Calendar::new();
    if let Some((y, m, d)) = crate::config::parse_date(&settings.string("goal-date")) {
        if let Ok(dt) = glib::DateTime::from_local(y, m as i32, d as i32, 12, 0, 0.0) {
            #[allow(deprecated)]
            calendar.select_day(&dt);
        }
    }

    let clear = gtk::Button::with_label("清除日期");
    clear.add_css_class("destructive-action");

    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 8);
    box_.set_margin_top(8);
    box_.set_margin_bottom(8);
    box_.set_margin_start(8);
    box_.set_margin_end(8);
    box_.append(&calendar);
    box_.append(&clear);

    let popover = gtk::Popover::new();
    popover.set_child(Some(&box_));

    let button = gtk::MenuButton::new();
    button.add_css_class("flat");
    button.set_tooltip_text(Some("从日历选择目标日期"));
    button.set_popover(Some(&popover));
    row.add_suffix(&button);
    row.set_activatable_widget(Some(&button));

    // 「清除日期」会在内部调 select_day()，那会触发 day_selected；
    // 该路径自己负责 popdown，用此标志避免重复排程。
    let suppress_popdown = Rc::new(Cell::new(false));

    let settings_c = settings.clone();
    let button_c = button.clone();
    let popover_c = popover.clone();
    let suppress_c = suppress_popdown.clone();
    calendar.connect_day_selected(move |cal| {
        let dt = cal.date();
        let text = format!(
            "{:04}-{:02}-{:02}",
            dt.year(),
            dt.month(),
            dt.day_of_month()
        );
        let _ = settings_c.set_string("goal-date", &text);
        button_c.set_label(&format_date_cn(&text));
        if suppress_c.get() {
            return;
        }
        // 选完即收起。day_selected 在点击的同一次事件处理中触发，
        // 立刻 popdown 会与 Calendar 内部的按压处理抢时序，故延后一拍。
        let popover = popover_c.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(150), move || {
            popover.popdown();
        });
    });

    let settings_c = settings.clone();
    let button_c = button.clone();
    let calendar_c = calendar.clone();
    let popover_c = popover.clone();
    let suppress_c = suppress_popdown.clone();
    clear.connect_clicked(move |_| {
        suppress_c.set(true);
        let _ = settings_c.set_string("goal-date", "");
        button_c.set_label("未设置");
        let today = glib::DateTime::now_local().unwrap_or_else(|_| {
            glib::DateTime::from_local(2026, 1, 1, 12, 0, 0.0).expect("valid date")
        });
        #[allow(deprecated)]
        calendar_c.select_day(&today);
        suppress_c.set(false);
        popover_c.popdown();
    });

    let saved = settings.string("goal-date").to_string();
    button.set_label(&format_date_cn(&saved));
    row
}

fn format_date_cn(raw: &str) -> String {
    match crate::config::parse_date(raw) {
        Some((y, m, d)) => format!("{y}年{m}月{d}日"),
        None => "未设置".to_string(),
    }
}

fn entry_row(settings: &Settings, key: &str, title: &str, placeholder: &str) -> adw::EntryRow {
    let row = adw::EntryRow::new();
    row.set_title(title);
    row.set_show_apply_button(true);
    // AdwEntryRow has no placeholder-text property on libadwaita 1.9 —
    // set_property on a missing key panics. Tooltip is the safe hint.
    row.set_tooltip_text(Some(placeholder));
    row.set_property("text", settings.string(key).as_str());
    let settings = settings.clone();
    let key = key.to_string();
    row.connect_apply(move |row| {
        let text: String = row.property("text");
        let _ = settings.set_string(&key, &text);
    });
    row
}

fn next_phase_preview(timer: &Timer) -> String {
    let next = match timer.phase {
        Phase::Work => {
            if timer.completed_work > 0
                && (timer.completed_work + 1) % timer.config.long_break_interval == 0
            {
                Phase::LongBreak
            } else {
                Phase::ShortBreak
            }
        }
        Phase::ShortBreak | Phase::LongBreak => Phase::Work,
    };
    let mins = match next {
        Phase::Work => timer.config.work_minutes,
        Phase::ShortBreak => timer.config.short_break_minutes,
        Phase::LongBreak => timer.config.long_break_minutes,
    };
    format!("{} {mins} 分钟", next.title())
}
