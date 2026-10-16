# Pomato

![DeepSeek Assisted Development](https://img.shields.io/badge/DeepSeek%20Assisted-Development-brightgreen?style=for-the-badge&logo=deepseek)

番茄钟（**Pom**odoro + To**mato**），面向 GNOME / Linux 桌面的 GTK4 原生计时器。

常驻托盘，关闭窗口也继续在后台计时。带倒数日总目标与支持自定义背景图的全屏专注界面。

![Pomato 图标](data/icons/pomato.png)

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
- 背景显隐由窗口真实 `fullscreened` 状态驱动 —— 用 Super 键等窗口管理器方式退出全屏也不会残留

### 其他
- 设计遵循 GNOME HIG，使用 libadwaita
- 配置存 GSettings（dconf），升级/重启不丢失

---

## 依赖

**运行**：GTK 4.22+、libadwaita 1.9+

**构建**（Fedora 44）：

```bash
sudo dnf install gtk4-devel libadwaita-devel rust cargo cargo-generate-rpm
```

推荐同时安装托盘扩展：

```bash
sudo dnf install gnome-shell-extension-appindicator
```

---

## 构建与安装

```bash
# 开发构建
cargo build

# 运行
cargo run

# 测试
cargo test

# 发布构建 + 打包 RPM
cargo build --release
cargo generate-rpm
# → target/generate-rpm/pomato-<版本>-1.x86_64.rpm
```

安装：

```bash
sudo rpm -Uvh target/generate-rpm/pomato-<版本>-1.x86_64.rpm
pkill -x pomato; nohup /usr/bin/pomato >/dev/null 2>&1 &
```

> ⚠️ **升级后必须先杀掉旧进程再启动**，否则运行的还是旧二进制。

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

仓库内提供了现成的恢复脚本 `dist/restore-pomato-settings.sh`，以及一份备份数据 `dist/pomato-settings-backup.ini`（两者角色不同，见 [`docs/CONFIGURATION.md`](docs/CONFIGURATION.md)）。

完整的 16 个配置键说明见 [`docs/CONFIGURATION.md`](docs/CONFIGURATION.md)。

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
| `REPORT-TO-PARENT.md` | ⏳ **一次性汇报**（非长期文档），向工作区统一管理 agent 说明当前冻结状态与待决策事项 |

---

## 当前状态

- 版本：**0.1.11**（已安装到系统）
- ⚠️ **工作树领先已安装版本**：0.1.12 的全屏状态同步修复已写入 `src/app.rs`，但**未 bump 版本、未打包、未安装**（用户明确要求先不打包）。因此系统当前运行的仍是未修复的 0.1.11。
- **仓库已初始化**（分支 `master`），远端主机 `github.com`、仓库路径 `SyntaxEntity/Pomato`（SSH）；**尚未推送任何内容**。

---

## 许可证

GPL-3.0-or-later，见 [`LICENSE`](LICENSE)。

---

## DeepSeek Assisted Development

![DeepSeek Assisted Development](https://img.shields.io/badge/DeepSeek%20Assisted-Development-brightgreen?style=for-the-badge&logo=deepseek)

本仓库代码由 **DeepSeek** 辅助编写（vibecoding）。AI 生成的代码可能包含缺陷、过时用法或不完整实现，
请自行评估后使用与集成。
