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

发版由 **CI 完成**：推 tag → GitHub Actions 在 Fedora 44 容器里跑门禁、构建 RPM、
创建 Release 并挂上资产。本机打包只用于「发版前给用户先验收」。

### 3.1 准备（在 master 上）

```bash
cd ~/Projects/Pomato

# 1. 改代码
# 2. 【必做】bump 版本号 —— 漏 bump 的话 dnf 不认为是升级
grep -m1 '^version' Cargo.toml     # 改这里
#    Cargo.lock 由 cargo 自动跟随，一起提交
#    README 的版本行 + 安装文件名也要同步（readme-lint 会检查一致性）
#    AGENTS.md §0 的版本行

# 3. 写 Release 说明，必须含「已知边界」
#    .github/release-notes/v<ver>.md     ← CI 会用它作为 Release 正文

# 4. 门禁（CI 会再跑一遍，本地先过一遍省时间）
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
~/Projects/Atlas/bin/readme-lint.py --repo . --online

# 5. 推送 master
git push origin master
```

### 3.2 可选：先给用户一个本地包验收

**未经验收的改动不要打 tag。** 需要用户先装上确认时，本地打一个包给他：

```bash
cargo build --release && cargo generate-rpm
# → target/generate-rpm/pomato-<ver>-1.x86_64.rpm
# 交用户执行（agent 无 sudo）：
#   sudo dnf install -y <上面的包>
#   pkill -x pomato; nohup /usr/bin/pomato >/dev/null 2>&1 &
```

### 3.3 打 tag 触发发版

```bash
# 【重要】tag 会冻结它指向的提交里的 workflow 文件。
# 改过 .github/workflows/release.yml 的话，必须先推送它，再打 tag。
git tag -a v<ver> -m "Pomato v<ver> — <一句话>"
git push origin v<ver>
```

CI 的 `release` workflow 会：

1. **verify**：校验 tag 与 `Cargo.toml` 版本一致 → fmt / clippy `-D warnings` / test
2. **publish**：`cargo build --release --locked` → `cargo generate-rpm` → 校验包内版本 → 创建 Release 并上传 RPM

任一步失败都不会产生 Release。可先手动触发一次 `workflow_dispatch`（`dry_run=true`，
只构建不发布）验证流水线：

```bash
gh workflow run release.yml --repo SyntaxEntity/Pomato -f dry_run=true
gh run watch --repo SyntaxEntity/Pomato
```

### 3.4 发版后核对

```bash
gh release view v<ver> --json tagName,isDraft,assets \
  --jq '"tag=\(.tagName) draft=\(.isDraft)", (.assets[] | "  \(.name)  \(.size) B")'
grep -c "已知边界" .github/release-notes/v<ver>.md   # 必须 ≥ 1
rpm -q pomato                                        # 与 tag 版本一致
```

> 🔴 **发版前必须获得用户同意。** 未验收通过的改动**不要**打 tag。

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

> **打包只有一条路径：`cargo generate-rpm`**（读 `Cargo.toml` 的 `[package.metadata.generate-rpm]`）。
> 早期的 `dist/pomato.spec` 已于 2026-10-17 删除 —— 它不在打包路径上，且版本长期漂移（`0.1.0`），是典型的「第二个真相源」。

---

## 6. 版本历史与产物

- 历史 RPM 都在 `target/generate-rpm/`：0.1.0 → 0.1.14
- 系统当前安装：`pomato-0.1.12-1.x86_64`
- **0.1.12**：全屏状态同步、日期弹窗自动收起 —— **用户真机验收通过** ✅
- **0.1.13**：窗口最小尺寸第一版（`set_size_request`）—— ❌ **实测无效**（Overlay 无主 child 导致 min 归零）
- **0.1.14 已构建**（2026-10-20）：`target/generate-rpm/pomato-0.1.14-1.x86_64.rpm`
  - 修正：内容改为 `gtk::Overlay` 主 child，窗口最小尺寸实测 **578×745**
  - 默认窗口尺寸 560×720 → 620×780（内容真实最小值 578×699 更大）
  - 已通过真实运行验证（临时探针），**待用户安装确认**
  - **未打 tag、未发 Release**

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
