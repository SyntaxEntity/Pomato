# 构建、打包与发版

---

## 1. 环境要求

- **Host**：Fedora 44 x86_64 GNOME（Wayland）
- **工具链**：rustc / cargo 1.97.1
- **已装 devel 包**：gtk4、libadwaita-1、webkitgtk-6.0、gtksourceview5
- **打包工具**：`cargo-generate-rpm`，位于 `~/.cargo/bin`
- 缺失时安装：

  ```bash
  sudo dnf install gtk4-devel libadwaita-devel rust cargo cargo-generate-rpm
  ```

> ⚠️ **agent 不能 sudo**（需要密码/指纹）。凡是提权操作，一律把完整命令打印给用户，由用户执行。

---

## 2. 日常开发

```bash
cd ~/Projects/Pomato

cargo build            # 开发构建
cargo run              # 直接运行
cargo test             # 单元测试，必须 8/8 全绿
cargo clippy           # 已知 8 条既有警告，非新增即可
```

---

## 3. 发版全流程（每次照做）

```bash
cd ~/Projects/Pomato

# 1. 改代码
# 2. 【必做】bump Cargo.toml 的 version
#    —— 不 bump 的话 rpm -Uvh / dnf 不认为是升级，会拒绝或静默不更新
# 3. 测试 + 发布构建
cargo test
cargo build --release
# 4. 打包
cargo generate-rpm
#    → target/generate-rpm/pomato-<ver>-1.x86_64.rpm
# 5. 把下面两行交给用户执行（agent 无 sudo）
#    sudo rpm -Uvh target/generate-rpm/pomato-<ver>-1.x86_64.rpm
#    pkill -x pomato; nohup /usr/bin/pomato >/dev/null 2>&1 &
```

> 🔴 **发版前必须获得用户同意。** 当前项目处于版本冻结状态（见 [`../AGENTS.md`](../AGENTS.md) 第 2 节），用户明确要求「先不要打包」。

---

## 4. ⚠️ stale-process 陷阱（真实踩过）

**安装/升级后必须杀掉旧进程再启动**：

```bash
pkill -x pomato; nohup /usr/bin/pomato >/dev/null 2>&1 &
```

否则用户看到的仍然是**旧二进制**的界面。真实事故：用户曾报告「根本没有全屏功能」，排查后发现是旧进程仍在运行，新版本其实已经装好。

同理，让用户「重启应用验证」时，务必把 `pkill` 一起给出，不要只说「重新打开」。

---

## 5. RPM 内容定义

打包资产定义在 `Cargo.toml` 的 `[package.metadata.generate-rpm]`：

| 源 | 目标 |
|---|---|
| `target/release/pomato` | `/usr/bin/pomato`（mode 755） |
| `data/io.github.pomato.Pomato.desktop` | `/usr/share/applications/` |
| `data/io.github.pomato.Pomato.gschema.xml` | `/usr/share/glib-2.0/schemas/` |
| `data/io.github.pomato.Pomato.service` | `/usr/share/dbus-1/services/` |
| `data/icons/hicolor/{32,48,64,128,256}x{sz}/apps/pomato.png` | `/usr/share/icons/hicolor/<sz>x<sz>/apps/` |

`post_install_script` / `post_uninstall_script` 会自动执行：`glib-compile-schemas`、`gtk-update-icon-cache`、`update-desktop-database`（各自带 `-x` 存在性检查与 `|| :` 容错）。

> `dist/pomato.spec` 存在，但**不是主路径**——主路径是 `cargo generate-rpm`。

---

## 6. 版本历史与产物

- 历史 RPM 都在 `target/generate-rpm/`：0.1.0 → 0.1.11（共 12 个）
- 系统当前安装：`pomato-0.1.11-1.x86_64`（安装于 2026-10-15 23:09）
- **0.1.12 尚未打包**（版本冻结中）

```bash
# 查看已安装版本与安装时间
rpm -q pomato
rpm -q --queryformat '%{INSTALLTIME:date}\n' pomato
```

---

## 7. 发版自测清单

打包交付前，至少手动覆盖以下各项：

- [ ] 窗口模式外观与改动前一致
- [ ] F11 进全屏 → 背景图 / 遮罩 / 模糊 / 遮罩颜色均按配置生效
- [ ] Escape 退出全屏干净，无背景残留
- [ ] 用窗口管理器方式（Super 键等）退出全屏，背景不残留
- [ ] 关闭窗口 → 进托盘，计时仍在后台走
- [ ] 托盘菜单各项可用（显示、开始/暂停、重置、跳过、退出）
- [ ] 改偏好设置 → 重启应用后仍生效（`dconf dump /io/github/pomato/` 复核）
- [ ] `cargo test` 8/8 全绿

> 无隔离显示环境时（例如 agent 运行在用户的实时 Wayland 会话中），GUI 行为**无法自动验证**，必须由用户真机验收。详见 [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md) 第 1 条。

---

## 8. 分发说明

- **RPM 是唯一分发渠道。**
- **Flatpak 明确暂缓**（用户决定），不要主动做。
- 目前**没有 git 仓库、没有 GitHub 远程**；`Cargo.toml` 里的 `repository` 字段是占位值 `https://github.com/pomato/pomato`，与实际账号不符，建仓后需修正。
