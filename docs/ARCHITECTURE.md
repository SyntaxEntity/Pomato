# 架构与实现细节

> 面向要改代码的人。入手前请先读 [`../AGENTS.md`](../AGENTS.md) 了解红线。

---

## 1. 技术栈（已锁定）

| 项 | 选择 |
|---|---|
| 语言 | Rust（edition 2024） |
| UI | **Relm4 0.11** + gtk4 0.11（feature `gnome_50`）+ libadwaita 0.9（feature `v1_9`） |
| 系统库 | Fedora 44：gtk4 4.22.5 / libadwaita 1.9.4 / glib 2.88.4 / rustc 1.97.1 |
| 计时 | `glib::timeout_add_seconds_local(1)` —— **不用 tokio** |
| 托盘 | **ksni 0.2**（StatusNotifierItem）+ flume 通道汇入 Relm4 `Msg::Tray` |
| 配置 | **GSettings**（不是 TOML/JSON） |
| 分发 | **RPM**（`cargo generate-rpm`）；Flatpak 明确暂缓 |
| 通知 | notify-rust |

**Dual-glib 陷阱**：glib 必须经 `gtk::gio` / `gtk::glib` 再导出使用。**禁止**并行引入 glib 0.21 crate（与 gtk4 依赖的 glib 0.22 冲突，会产生两套不兼容类型，报错极难读）。

---

## 2. 目录结构

```
Pomato/
├── Cargo.toml          # 版本号在此；package.metadata.generate-rpm 定义 RPM 资产与安装脚本
├── build.rs            # 把 GSettings schema 编译到 OUT_DIR（开发期用）
├── LICENSE             # GPL-3.0-or-later
├── README.md           # 用户/协作者向
├── AGENTS.md           # agent 上手契约（红线）
├── REPORT-TO-PARENT.md # 向上级管理 agent 的一次性汇报（非长期文档）
├── docs/               # 本目录：细节文档
├── src/
│   ├── main.rs         # 入口：adw::init、安装全局 CSS、RelmApp::run
│   ├── app.rs          # Relm4 UI：主窗口、偏好设置、全屏背景、日历选日期
│   ├── config.rs       # Config 结构、GSettings 读取、日期解析、倒数日文案
│   ├── timer.rs        # 状态机：阶段、tick、Start/Pause/Reset/Skip
│   ├── notifier.rs     # 提示音 + notify-rust 桌面通知
│   └── tray.rs         # ksni 托盘（SNI）
├── data/
│   ├── io.github.pomato.Pomato.gschema.xml
│   ├── io.github.pomato.Pomato.desktop
│   ├── io.github.pomato.Pomato.service   # D-Bus activation 服务文件
│   ├── style.css                          # 通过 include_str! 编译进二进制
│   └── icons/…                            # 多尺寸 hicolor PNG + pomato.png 512 master
├── dist/
│   ├── pomato.spec                     # 存在但不是主路径，generate-rpm 才是
│   ├── pomato-settings-backup.ini      # dconf 设置备份
│   └── restore-pomato-settings.sh      # dconf load 恢复
└── target/                             # 构建产物；target/generate-rpm/ 存历史 RPM
```

代码规模：约 1550 行 Rust（`app.rs` 最大，约 940 行）。

---

## 3. 模块职责与数据流

```
                 ┌──────────────────────────────────────┐
   GSettings ────┤ connect_changed → Msg::ConfigChanged  │
                 └───────────────────┬──────────────────┘
                                     ▼
   glib timeout(1s) ──► Msg::Tick ──► Timer::tick()
                                     │
   UI 按钮 ───────────► Msg::Toggle / Reset / Skip
                                     │
   ksni 托盘 ──flume──► Msg::Tray(TrayEvent)
                                     │
   window ──notify::fullscreened ────► Msg::FullscreenChanged(bool)
                                     ▼
                          App::update()  →  改 Timer 状态
                                          →  progress Cell 更新
                                          →  DrawingArea::queue_draw()
                                          →  sync_tray()
```

- **`timer.rs`**：纯状态机，不碰 UI。持有 `phase`、`status`、`remaining`、`completed_work`。派生数据：`display_time()`、`progress()`、`session_label()`、`tray_title()`、`phase_duration_secs()`。
- **`config.rs`**：`Config` 结构 + `from_settings()` 读取 + 日期解析（`YYYY-MM-DD`）+ 倒数日文案（`goal_label()`）。含 4 个单元测试。
- **`app.rs`**：唯一的 Relm4 `SimpleComponent`。视图全在一个 `view!` 块里，`update()` 是唯一状态写入点。持有 `window` / `ring` / `bg_picture` / `bg_scrim` / `bg_css` 等控件句柄。
- **`tray.rs`**：ksni `Tray` 实现 + `TrayHandle`（`set_title` / `set_running` / `set_icon_name`）+ flume 事件接收。托盘菜单项发出 `TrayEvent`。
- **`notifier.rs`**：`beep()`（目前是 `gdk::Display::beep()`）与 `phase_finished()`（notify-rust）。

**托盘可用性语义（两个方向都要记住，容易只记一半）**：

- 托盘**不存在**（缺 `gnome-shell-extension-appindicator`）→ **不许杀进程**。应用保持常驻，并发通知提示用户安装扩展。
- 关窗行为由 `Msg::HideWindow` 决定，实际条件是 `(close_to_tray || minimize_to_tray) && show_tray_icon`：
  - 成立 → 窗口 `set_visible(false)`，计时继续在后台跑
  - 不成立（两个开关都关，或 `show-tray-icon` 关）→ **直接 `Msg::Quit`，退出进程**
  - 注意该判断只看**配置开关**，并不检查托盘是否真的注册成功

**进度环**：不为每帧存 UI 状态，而是用 `Rc<Cell<f64>>` 共享当前进度值，`DrawingArea::set_draw_func` 闭包直接读该 Cell，靠 `queue_draw()` 触发重绘。画布 340×340。

---

## 4. 全屏与自定义背景（0.1.12 起的设计）

### 唯一真相源原则

**全屏状态只有一个真相源：窗口的 `fullscreened` 属性。**

历史 bug（0.1.11 及以前）：`App` 里维护了一个 `fullscreen: bool` 镜像字段，只在自家的 `Msg::ToggleFullscreen` / `Msg::ExitFullscreen` 里翻转。用户若用窗口管理器方式退出全屏（Super 键、拖拽、热角），flag 不会更新，于是背景图/遮罩残留、F11 方向判断错乱。

**修法不是同步两个状态，而是删掉其中一个**：

| 机制 | 实现 |
|---|---|
| 状态读取 | `window.is_fullscreen()`（需要判断时现查） |
| 变更通知 | `root.connect_fullscreened_notify(...)` → `sender.input(Msg::FullscreenChanged(window.is_fullscreen()))` |
| 应用背景 | `apply_fullscreen_bg(&self, fullscreened: bool)` —— 状态由参数传入，不读字段 |
| `Msg::ToggleFullscreen` | 查 `is_fullscreen()` 决定调 `fullscreen()` 还是 `unfullscreen()`，**不写状态** |
| `Msg::ExitFullscreen` | 无条件 `unfullscreen()`（幂等，非全屏时 no-op），**不写状态** |

因为 `notify::fullscreened` 是 GTK 属性通知，任何来源的状态变化都会触发它，所以合成器侧的改变也能被正确响应。

> ⚠️ **禁止**未来再给 `App` 加回任何形式的全屏镜像字段。要判断就现查 `is_fullscreen()`，要响应就用 notify。

### 视图层实现

View 层级（`view!` 块内）：

```
adw::ApplicationWindow
└─ adw::ToolbarView
   ├─ add_top_bar  → #[name = "header"] adw::HeaderBar（标题 + 齿轮 + 全屏按钮）
   └─ set_content  → gtk::Overlay
      ├─ #[name = "bg_picture"] gtk::Picture   # ContentFit::Cover，默认隐藏
      ├─ add_overlay → #[name = "bg_scrim"] gtk::Box  # 遮罩，默认隐藏
      └─ add_overlay → 内容 Box（阶段标题、进度环+时间、会话标签、控制按钮、目标行）
```

**全屏排版细节**：进度环 `DrawingArea` 画布 340×340，时间文字叠在其上（`gtk::Overlay` + 居中 `Label`，CSS class `.timer-display`；`data/style.css` 里 `font-size: 64pt`、`font-weight: 300`、`font-feature-settings: "tnum"` 等宽数字）。

- 背景图与遮罩**只在全屏且 `fullscreen-bg-path` 非空**时 `set_visible(true)`。
- 遮罩不透明度用 `scrim.set_opacity(cfg.fullscreen_bg_dim)`。
- 模糊与遮罩颜色**不能写在静态 `style.css`**（值是用户可配的），改用运行时注入：

  ```rust
  provider.load_from_string(&format!(
      "picture.bg-picture {{ filter: blur({blur:.1}px); }}\n\
       box.bg-scrim {{ background-color: {color}; }}"
  ));
  ```

  `CssProvider` 在 `init` 时以 `STYLE_PROVIDER_PRIORITY_USER` 挂到 display，句柄存在 `App.bg_css`。

**已知边界**：进入全屏后的显隐完全依赖 notify，没有再补一次「乐观 apply」。若将来在某个合成器上出现背景延迟一拍，可在 `ToggleFullscreen` 里补一次乐观调用。

---

## 5. 代码层已知坑（踩过再记录）

### Relm4 `view!` 宏
- 控件属性用 `=` 不是 `:`。
- 类型是 `Option<&impl IsA<Widget>>` 的属性要写 `#[wrap(Some)] name = &Widget {}`。
- `add_overlay = &widget {}` **不要**套 `#[wrap(Some)]`。
- `#[name = "..."]` 写在 `add_overlay = &Box { … }` **内部**会报「Properties can only have watch…」——必须写在 `add_overlay` **之前**。
- `DrawingArea::set_draw_func` 的闭包参数 `w, h` 是 `i32`，用时需转 `f64`。

### 所有权与类型
- **`Config` 含 String 字段 → 只实现了 `Clone`**：`self.config` 直接拷贝会编译错误，必须 `.clone()`。
- **`adw::PreferencesPage` / `PreferencesGroup`** 没有 struct 字段式初始化，用 `::new()` + setter（如 `set_title()`、`set_description()`）。
- **`ColorDialogButton::new(Option<ColorDialog>)` 按值传**：要 `Some(dialog.clone())`，不是 `Some(&dialog)`。
- **ksni 0.2.2**：`TrayService::new(tray)` 之后再 `spawn()` / `spawn_without_dbus_name()`——这俩是 `TrayService` 的方法，不是 `Tray` 的。

### 打包与集成
- **desktop 文件**：写 `DBusActivatable=true` 就**必须**同时提供 D-Bus service 文件，否则点图标静默失败。当前 `Exec` 用绝对路径 `/usr/bin/pomato`。
- **GSettings schema**：开发期走 `POMATO_SCHEMA_DIR`（`build.rs` 编译到 `OUT_DIR`）；安装后走系统 schema。**改 gschema 后必须重新 build 才生效。**

### libadwaita 0.9 的缺失
- **没有** `AdwCalendar` / `AdwCalendarRow` → 日期选择用 `gtk::Calendar`。
- **没有** EntryRow 的 `placeholder-text` → 只能用 tooltip 代替。

### `gtk::Calendar` 用法要点
- `date() -> DateTime`
- `set_date(&DateTime)`（v4_20 起的名字；`select_day` 是旧名，现为 deprecated）
- `connect_day_selected`
- `DateTime::from_local(y, m, d, h, mi, s) -> Result`
- `month()` / `day_of_month()` 返回 `i32`

### 图标（重要，易搞错）
当前图标（0.1.9+）是**第一版手绘简笔番茄**：用 Pillow `ImageDraw` 画的亮红椭圆果 + 高光 + 绿蒂/叶。

**历史四版谱系（勿搞错）**：

| 序 | 图标 | 曾随版本 |
|---|---|---|
| ① | 简笔番茄（**当前**） | 0.1.9+ |
| ② | GNOME 低饱和圆卡 | 0.1.0 / 0.1.1 |
| ③ | 表盘 SVG 番茄 | 短暂见于 0.1.8 |
| ④ | Google Noto emoji | 0.1.2 – 0.1.7 |

- 用户说「你自己画的」= agent 手绘稿；说「再往上两个版本」= 沿这条手绘谱系往回走。
- 产物：`data/icons/pomato.png`（512 master）+ `data/icons/hicolor/{32,48,64,128,256}x{sz}/apps/pomato.png`。
- ⚠️ `data/icons/pomato.svg` 仍在树里但**不是**当前图标源，**别拿它去 regenerate**。可删或加注释标明遗留。

---

## 6. 测试

`cargo test` 共 **8 个**单元测试，必须保持全绿：

| 模块 | 测试 | 覆盖 |
|---|---|---|
| `config` | `parse_valid_date` | 合法日期解析 |
| `config` | `parse_rejects_bad_dates` | 非法日期拒绝 |
| `config` | `days_since_epoch` | 天数差计算 |
| `config` | `goal_label_states` | 倒数日各种文案分支 |
| `timer` | `work_then_short_break` | 专注 → 短休流转 |
| `timer` | `long_break_after_interval` | 达标后进长休 |
| `timer` | `reset_restores_phase_duration` | 重置恢复本阶段时长 |
| `timer` | `skip_advances_without_counting_work` | 跳过不计入完成数 |

**测试空白**：倒数日 / 日期 UI、全屏背景、托盘均无自动化测试（纯 UI 逻辑）。GUI 行为只能靠真机手动验收。

---

## 7. 代码风格现状

- 项目存在**历史未格式化代码**（`cargo fmt --check` 会报若干处，如 `prefer_tray` 绑定、若干 `switch_row` 调用被折行）。**改动时不要把 `cargo fmt` 全文件跑一遍**，那会制造大片无关 diff。
- `cargo clippy` 有 8 条既有警告（unnecessary cast、let-binding unit value、collapsible if、`manual_is_multiple_of`）。属既有技术债，非新引入。
- `app.rs` 中 `switch_row` 函数签名后第一条语句与签名挤在同一行（格式瑕疵，见 [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md)）。
