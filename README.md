# Pomato

![Rust](https://img.shields.io/badge/Rust-1.94%2B-blue?style=for-the-badge&logo=rust)
![GTK](https://img.shields.io/badge/GTK-4.22-blue?style=for-the-badge&logo=gtk)
![libadwaita](https://img.shields.io/badge/libadwaita-1.9-blue?style=for-the-badge&logo=gnome)
![license](https://img.shields.io/badge/license-GPL--3.0--or--later-blue?style=for-the-badge)
![DeepSeek Assisted Development](https://img.shields.io/badge/DeepSeek%20Assisted-Development-brightgreen?style=for-the-badge&logo=deepseek)

番茄钟（**Pom**odoro + To**mato**），面向 GNOME / Linux 桌面的 GTK4 原生计时器。

常驻托盘，关闭窗口也继续在后台计时；带倒数日总目标与支持自定义背景图的全屏专注界面。

> 版本 `0.1.18` ｜ 状态：可用（开发中）｜ 已知限制见 [§已知限制](#已知限制)

---

## 功能

### 番茄钟核心
- 三种阶段：**专注 / 短休 / 长休**；每完成 N 个专注进入长休（默认 4 个，可配）
- 控制：开始 / 暂停 / 恢复 / 重置 / 跳过
- 时长可配（分钟）：专注、短休、长休、长休间隔
- 行为开关：自动开始下一阶段、提示音、桌面通知
- 阶段结束自动切换，可配置自动开始下一阶段

### 托盘常驻
- 始终注册 StatusNotifierItem（SNI）托盘图标
- 关闭窗口 / 最小化 → 收进托盘，**计时不中断**
- 托盘菜单：显示窗口、开始/暂停、重置、跳过、退出
- **退出只走托盘「退出」**（无 Ctrl+Q 快捷键）
- 未安装 AppIndicator 扩展时不会崩溃，会发通知提示安装 `gnome-shell-extension-appindicator`

### 总目标（倒数日）
- 自定义目标名称 + 目标日期，例如「项目截止 还有 64 天」
- ⚠️ 这里的天数是**距目标日期的倒数日**，**不是**番茄周期的剩余时间
- 日期用**日历弹窗**选择（`gtk::Calendar` + `MenuButton`/`Popover`），存储格式 `YYYY-MM-DD`
- 文案随日期自动变化：`还有 N 天` / `今天是 X` / `X 已过去 N 天` / `未设置目标` / `日期无效`
- 一键清除日期

### 全屏专注模式
- F11 切换 / Escape 退出 / 标题栏按钮
- 环形进度条（cairo 绘制，340×340）+ 64pt 大号时间显示
- **自定义背景**（0.1.10+）：背景图 + 可调遮罩颜色/不透明度 + 高斯模糊
- 背景只在全屏时生效，窗口模式保持 GTK 原生外观

### 其他
- 设计遵循 GNOME HIG，使用 libadwaita
- 配置存 GSettings（dconf），升级/重启不丢失

![Pomato 图标](data/icons/pomato.png)

---

## 安装

**当前没有发布包渠道**（无 `.rpm` 发布页、无 Flatpak、无发行版仓库收录）。仓库已开源，但**尚未创建 Release**，请从源码构建后安装。

### 从源码构建并安装

先装构建依赖（Fedora 44）：

```bash
sudo dnf install gtk4-devel libadwaita-devel rust cargo cargo-generate-rpm
```

推荐同时安装托盘扩展：

```bash
sudo dnf install gnome-shell-extension-appindicator
```

构建并生成 RPM：

```bash
cargo build --release
cargo generate-rpm
# → target/generate-rpm/pomato-0.1.18-1.x86_64.rpm
```

安装：

```bash
sudo rpm -Uvh target/generate-rpm/pomato-0.1.18-1.x86_64.rpm
```

> ⚠️ **升级后必须先杀掉旧进程再启动**，否则运行的还是旧二进制：
> ```bash
> pkill -x pomato; nohup /usr/bin/pomato >/dev/null 2>&1 &
> ```

完整发版流程见 [`docs/BUILD-AND-RELEASE.md`](docs/BUILD-AND-RELEASE.md)。

---

## 使用

| 操作 | 方式 |
|---|---|
| 开始 / 暂停 | 主界面按钮，或托盘菜单 |
| 重置当前阶段 | 主界面按钮 |
| 跳过当前阶段 | 主界面按钮 |
| 进入 / 退出全屏 | `F11`，或标题栏全屏按钮 |
| 退出全屏 | `Escape` |
| 偏好设置 | 标题栏齿轮按钮 |
| 关闭到托盘 | 点窗口关闭按钮（默认行为） |
| **退出程序** | **托盘图标 → 退出**（唯一方式） |

---

## 配置

所有设置存在 GSettings（schema `io.github.pomato.Pomato`，路径 `/io/github/pomato/`），可在偏好设置里修改，也可用命令行：

```bash
# 查看全部设置
dconf dump /io/github/pomato/

# 备份
dconf dump /io/github/pomato/ > pomato-settings.ini

# 恢复
dconf load /io/github/pomato/ < pomato-settings.ini
```

仓库内提供一份**脱敏模板** [`packaging/pomato-settings.example.ini`](packaging/pomato-settings.example.ini)，含全部 16 个键的占位值，可直接导入：

```bash
dconf load /io/github/pomato/ < packaging/pomato-settings.example.ini
```

完整的 16 个配置键说明见 [`docs/CONFIGURATION.md`](docs/CONFIGURATION.md)。

---

## 构建（开发）

```bash
# 开发构建
cargo build

# 直接运行（不安装）
cargo run

# 单元测试（8 个，必须全绿）
cargo test

# 发布构建 + 打包 RPM
cargo build --release
cargo generate-rpm
```

---

## 项目结构

```
Pomato/
├── src/
│   ├── main.rs      # 入口：adw::init、安装全局 CSS、RelmApp::run
│   ├── app.rs       # Relm4 UI：主窗口、偏好设置、全屏背景、日历选日期
│   ├── config.rs    # Config 结构、GSettings 读写、日期解析、倒数日文案
│   ├── timer.rs     # 状态机：阶段、tick、开始/暂停/重置/跳过
│   ├── notifier.rs  # 提示音 + 桌面通知
│   └── tray.rs      # StatusNotifierItem 托盘
├── data/            # GSettings schema、desktop/service 文件、style.css、图标
├── docs/            # 架构、配置、构建发布、已知问题、路线图
└── packaging/       # 打包配置与脱敏配置模板
```

---

## 技术栈

| 项 | 版本 / 选择 |
|---|---|
| 语言 | Rust（edition 2024，MSRV 1.94） |
| UI | GTK 4.22 + libadwaita 1.9 + Relm4 0.11 |
| 计时 | `glib::timeout_add_seconds_local(1)`（无异步运行时） |
| 托盘 | ksni 0.2（StatusNotifierItem） |
| 配置 | GSettings（dconf） |
| 分发 | RPM（`cargo generate-rpm`） |

**运行要求**：GTK 4.22+、libadwaita 1.9+。
**构建要求**：Rust **1.94+**（`Cargo.toml` 已声明 `rust-version`）。

---

## 已知限制

- **平台**：只在 **Fedora 44 x86_64 + GNOME（Wayland）** 上构建并运行过；其他发行版、其他桌面环境、X11 均**未验证**。
- **未验证**：Windows 与 macOS 未构建、未运行；本项目只面向 Linux。
- **托盘依赖外部 GNOME 扩展**：不使用 GNOME 时需自行提供 StatusNotifierItem 宿主。未安装 `gnome-shell-extension-appindicator` 时托盘图标不显示，应用不会崩溃但会常驻后台并提示安装。
- **全屏背景残留 bug（0.1.11 及以前）**：用窗口管理器方式退出全屏（Super 键、拖拽、热角）时，背景图/遮罩可能残留，且 F11 方向判断可能错乱。**0.1.12 已含修复**，但尚未经真机验证。
- **日期弹窗不自动收起（0.1.11 及以前）**：在日历里点选一天后，弹窗仍保持打开，需要手动关闭。**0.1.12 已含修复**，但尚未经真机验证。
- **无会话持久化**：计时状态与「第几个番茄」不落盘，应用重启或崩溃后倒计时、剩余秒数、已完成番茄数全部清零。配置（GSettings）不会丢。
- **无 i18n**：所有界面文案硬编码为中文，不支持其他语言。
- **提示音很弱**：目前是系统蜂鸣（`gdk::Display::beep()`），不是可配置音效，无音量/音色选项。
- **0.1.12 的两项修复尚未做运行态验证**：`Cargo.toml` 与 `README` 已标为 0.1.12，但全屏状态同步与日期弹窗自动收起这两个修复只在开发环境做过静态验证（编译/单测/审计），未经真机确认。
- **仅提供 RPM**：不提供 Flatpak、deb 或 AUR 包。

---

## 路线图

当前优先级（详见 [`docs/ROADMAP.md`](docs/ROADMAP.md)）：

- **P1**：全屏沉浸式（隐藏 HeaderBar）——需先确认产品预期
- **P2**：计时会话持久化；i18n；无障碍标注
- **P3**：快捷键扩展；自定义提示音

> 已完成的修复（待打包发布）：全屏状态同步、日期弹窗选完自动收起。

**明确不做**：云同步、多平台、插件系统、Flatpak（暂缓）、Ctrl+Q 快捷键（已移除，退出只走托盘）。

---

## 项目文档

| 文档 | 内容 |
|---|---|
| [`AGENTS.md`](AGENTS.md) | 给接手本项目的人/AI 的最短上手契约：硬性约束、工作协议、验证清单 |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | 模块职责、消息流、状态机、全屏与背景实现、Relm4 坑位 |
| [`docs/CONFIGURATION.md`](docs/CONFIGURATION.md) | GSettings 键位完整表 |
| [`docs/BUILD-AND-RELEASE.md`](docs/BUILD-AND-RELEASE.md) | 构建、打包、安装、发版全流程 |
| [`docs/KNOWN-ISSUES.md`](docs/KNOWN-ISSUES.md) | 已知问题与优先级 |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | 待办、已砍事项、明确不做的事 |

---

## 贡献 / 反馈

问题与建议请提交到 [GitHub Issues](https://github.com/SyntaxEntity/Pomato/issues)。仓库刚上线、暂无公开 issue，也可以先看 [`docs/KNOWN-ISSUES.md`](docs/KNOWN-ISSUES.md) 是否已有记录。

代码约定见 [`AGENTS.md`](AGENTS.md)：Rust + GTK4 + Relm4 技术栈锁定，改动后 `cargo test` 须保持 8/8 全绿。

---

## 许可证

**GPL-3.0-or-later**（SPDX：`GPL-3.0-or-later`），全文见 [`LICENSE`](LICENSE)。

---

## DeepSeek Assisted Development

![DeepSeek Assisted Development](https://img.shields.io/badge/DeepSeek%20Assisted-Development-brightgreen?style=for-the-badge&logo=deepseek)

本仓库代码由 **DeepSeek** 辅助编写（vibecoding）。AI 生成的代码可能包含缺陷、过时用法或不完整实现，
请自行评估后使用与集成。
