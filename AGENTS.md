# AGENTS.md — Pomato 上手契约

> 给接手本项目的 AI agent / 协作者。**先读完本文件再动手**，它只放红线与最短上手路径；细节都在 [`docs/`](docs/) 里，需要时再读。
>
> 本文件替代了旧的单文件 `HANDOFF.md`（已废弃删除）。

> **⚠️ 每轮开工先读 `~/Projects/Atlas/brief.md`** —— 那是 Atlas 的当前轮次派发单，
> 读完这一个文件就知道本轮该做什么、不该做什么。没有你的任务就回报「无新增任务」，
> **不要自行找活干**。派发单里的任务**幂等**：已完成就核对并跳过，不要重做。

---

## 0. 30 秒认知

**Pomato** = GNOME 番茄钟，Rust + GTK4 + libadwaita + Relm4，托盘常驻，RPM 分发。

- 根目录：`~/Projects/Pomato`（即本文件所在目录）
- application-id：`io.github.pomato.Pomato`
- 版本：`Cargo.toml` = **0.1.15**（待用户安装验收）；系统已装 `pomato-0.1.12-1.x86_64`
- **已开源**：`github.com/SyntaxEntity/Pomato`（`master`，SSH remote）· **尚无 Release / tag**
- 目标发行版基线：**Fedora 44**（GTK 4.22.5 / libadwaita 1.9.4 / rustc 1.97.1）

---

## 1. 动手前先核实现状（不许信文档当事实）

文档是快照，不是事实。**第一件事**：

```bash
cd ~/Projects/Pomato                   # 或直接在本文件所在目录执行
grep -m1 '^version' Cargo.toml      # 真实版本号
cargo test                          # 应为 8/8 全绿
cargo build                         # 应能编译
rpm -q pomato                       # 系统装的是哪个版本
pgrep -a -x pomato                  # 有没有在跑
```

已有过「把旧 checkpoint 的『没写完』当成现状」的乌龙，别重演。

---

## 2. 🔴 当前状态：0.1.17 已打包，等待用户安装验收

Atlas 裁定 **D-27**（用户 2026-10-20：「先打包，让用户装上看效果」）已解除版本冻结：

- `Cargo.toml` = **`0.1.17`**，RPM 已构建（见 `target/generate-rpm/`）
- **系统里装的是 `0.1.12`** —— 0.1.17 待安装
- **未打 tag、未发 Release**（等用户真机验收后再发版）

**0.1.12 的两项修复已经用户真机验收通过**（2026-10-20）：全屏状态同步 ✅、日期弹窗自动收起 ✅。
**0.1.14**：窗口最小尺寸 —— 此前窗口被拖得很小时控件会溢出到窗口外。
根因是 `gtk::Overlay` 没有主 child 导致最小尺寸归零（`0.1.13` 的修法无效，已在 0.1.14 纠正）。
实测窗口最小尺寸 **578×745**。
**0.1.15**：修窗口打开时右边缘的 1-2px 暗线（显式绘制窗口底色 + 首帧后强制重绘）。
实测内容宽度与窗口宽度相等（620/620），故该线来自窗口 CSD 边缘，不是布局留边。

**仍不要在未获授权时打 tag 或发 Release。**

---

## 3. 硬性约束（违反会被用户纠正）

1. **语言/栈锁定**：Rust + GTK4 + Relm4。不许擅自换语言、换 UI 框架。
2. **托盘常驻**：关闭/最小化进托盘，计时继续后台跑。
3. **退出只走托盘「退出」**。**Ctrl+Q 已被用户明确砍掉，禁止再加。**
4. **缺 AppIndicator 扩展时不许杀进程**——保持常驻并发通知提示安装扩展。**另一面也要记住**：若 `close-to-tray`/`minimize-to-tray` 都关、或 `show-tray-icon` 关，关窗会**直接退出进程**（见 [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) 第 3 节）。
5. **配置只走 GSettings**，不是 TOML/JSON。
6. **分发只走 RPM**；**每次发版必须 bump `Cargo.toml` 版本号**，否则 rpm/dnf 不认升级。
7. **基线 Fedora 44**，不要按 Fedora 45 / GTK 4.24 / libadwaita 1.10 规划。
8. **agent 不能 sudo**（需密码/指纹）：需要提权的操作一律打印完整命令交用户执行。
9. **禁止循环搜 API**：同一轮里同一个 grep 出现第二次就必须停下改代码。用户曾因「干啥呢循环上了」明确不满——历史事故：某次 PDF 调研同一 grep 重复了六轮。
10. **功能范围不擅自扩张**：i18n / a11y / 快捷键扩展属「计划中未做」；云同步 / 多平台 / 插件**明确不做**；Flatpak 用户**明确暂缓**。
11. **不擅自 `git init`**：需用户同意。
12. **不在用户实时桌面盲启 GUI**：无隔离显示环境时，GUI 改动只能静态验证 + 交用户真机验收。

---

## 4. 工作协议

- **最小改动**：能改一行不重构整个文件；不要顺手 `cargo fmt` 全文件（会制造大片无关 diff，项目本身有历史未格式化代码）。
- **不过度设计**：不引入新依赖来解决问题；不为未提出的需求写抽象层。
- **先修结构再修表象**：能靠「消除第二种状态」解决的，不要靠「同步两种状态」。
- **不夸大验证**：编译通过 ≠ 功能可用。没跑过就写「未做运行态验证」。
- **交付时要说明**：改了什么、验证到哪一步、还剩什么没验证。

---

## 5. 每次改动后的验证清单

```bash
cargo test                      # 8/8 必须全绿（config 4 + timer 4）
cargo build --release           # 必须编译通过
```

发版时（**需用户先同意**）额外执行：

```bash
# 1. bump Cargo.toml 的 version（必做！）
# 2. 打包
cargo generate-rpm
# 3. 交用户执行（agent 无 sudo）：
#    sudo rpm -Uvh target/generate-rpm/pomato-<ver>-1.x86_64.rpm
#    pkill -x pomato; nohup /usr/bin/pomato >/dev/null 2>&1 &
```

**安装后必须杀旧进程再启动**，否则用户看到的还是旧界面（真的坑过：用户曾报「根本没有全屏功能」，实为旧进程）。

手动自测至少覆盖：窗口模式外观不变、F11 全屏背景/遮罩/模糊/颜色生效、Escape 退出干净、托盘关窗后台仍计时、改设置后重启仍在（`dconf dump /io/github/pomato/`）。

---

## 6. 代码层坑位速查（细节见 docs/ARCHITECTURE.md）

- **全屏状态只有一个真相源**：窗口的 `fullscreened` 属性。**禁止**再给 `App` 加 `fullscreen: bool` 之类的镜像字段——那正是已修 bug 的成因。
- **glib 引用**：必须用 `gtk::gio` / `gtk::glib` 再导出，**禁止**并行引入 glib 0.21（与 gtk4 的 glib 0.22 冲突，Dual-glib trap）。
- **Relm4 `view!`**：属性用 `=` 不是 `:`；`add_overlay = &widget {}` 不要套 `#[wrap(Some)]`；`#[name = "..."]` 要写在 `add_overlay` **之前**。
- **`Config` 含 String → 只有 `Clone`**：`self.config` 不能直接拷贝，用 `.clone()`。
- **libadwaita 0.9 没有** `AdwCalendar`；日期选择用 `gtk::Calendar`；`EntryRow` 无 placeholder-text，只能用 tooltip。
- **GSettings schema**：开发走 `POMATO_SCHEMA_DIR`（build.rs 编到 OUT_DIR），改 gschema 后必须重新 build。
- **图标**：当前图标是 Pillow 脚本生成的手绘番茄 PNG；`data/icons/pomato.svg` 是**遗留文件，不是图标源**，别拿它 regenerate。

完整清单（含 ksni / ColorDialogButton / desktop 文件 / CSS 注入等）见 [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)。

---

## 7. 文档地图

| 文档 | 什么时候读 |
|---|---|
| [`README.md`](README.md) | 需要项目定位、功能清单、构建安装、用户视角说明 |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | 要改代码：模块职责、消息流、状态机、坑位全表 |
| [`docs/CONFIGURATION.md`](docs/CONFIGURATION.md) | 要加/改配置项 |
| [`docs/BUILD-AND-RELEASE.md`](docs/BUILD-AND-RELEASE.md) | 要发版 |
| [`docs/KNOWN-ISSUES.md`](docs/KNOWN-ISSUES.md) | 要挑活干、或排查 bug |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | 想确认某功能该不该做 |

**维护约定**：改代码后同步更新对应文档；本文件只放红线，细节请写进 `docs/`，不要让它重新膨胀成第二个 HANDOFF。

---

## 8. 汇报关系

本项目处于工作区 `~/Projects` 的统一管理 agent **Atlas**（`~/Projects/Atlas`）之下。跨项目事务（**GitHub 远程管理、仓库创建、发版节奏、需要用户提权的系统操作、跨项目约定**）由 Atlas 统筹。

- 项目内技术决策：本项目 agent 自主，无需请示。
- 需要上报：发版、远程仓库变更、跨项目约定、要改用户既有决策的事。
- 最新一次汇报已归档至 `~/Projects/Atlas/reports/inbox/AGENT-REPORT-Pomato-20261017-01.md`
  （原 `REPORT-TO-PARENT.md` 已迁走 —— ⏳ 一次性汇报，会随事务推进过期，别当长期文档维护）。

---

## 9. 工作区统一规范（Atlas 管辖）

**红线摘要见 `~/Projects/Atlas/prompts/PROMPT-00-workspace-norms.md`；唯一权威是 `~/Projects/Atlas/CONVENTIONS.md`。** 本节只列与 Pomato 直接相关的部分。

### 9.1 规范带来的两处变化（相对本文件既有内容）

1. **`AGENTS.md` 的定位已全区统一**：短、只放红线与索引，细节下沉 `docs/`。本项目现有结构**已符合**，无需调整。
2. **ssh 绕行方案作废**：本文件早期版本与本项目汇报中记录的
   `GIT_SSH_COMMAND='ssh -F /dev/null -i ~/.ssh/id_ed25519'`
   **不要再使用** —— `/etc/ssh/ssh_config.d/` 的属主故障曾于 2026-10-17 短暂复现，
   现已恢复正常：`ssh -T git@github.com` → `Hi SyntaxEntity! You've successfully authenticated`。
   密钥按 Host 绑定在 `~/.ssh/config`，直接 `git push` 即可。
   故障若复现 → 上报 Atlas，不要各自发明绕行方案。

### 9.2 建仓状态 ✅ 已完成（2026-10-17）

- **remote 用 SSH**：`git@github.com:SyntaxEntity/Pomato.git` ✅ 已配。禁止 HTTPS+token。
- **提交身份**：`SyntaxEntity <265272538+SyntaxEntity@users.noreply.github.com>` ✅ 已配
  —— **真实邮箱永不出现在提交里**。
- **隐私审计钩子**：✅ 已装。注意它走的是 `core.hooksPath`（不是 `.git/hooks/pre-push`），
  用 `~/Projects/Atlas/bin/install-hooks.sh --check` 核对，别用 `ls .git/hooks/` 判断。
- **`.gitignore`**：✅ 已建，含 `.dsh/`（本地私有工具目录）。
  **个人数据已彻底移出仓库**（Atlas 裁定 D-26/S3，2026-10-20）：
  `pomato-settings-backup.ini`（含**私人壁纸绝对路径**与**个人目标名称**）与
  `restore-pomato-settings.sh` 现在位于 `~/Projects/Pomato/.dsh/`，**不在仓库树内**。
  → 面向用户的脱敏模板：`packaging/pomato-settings.example.ini`（全占位值，入库）。
- **仓库 description**：✅ 已设，不带 `[Vibe]` 前缀。
- **首推**：✅ 已完成（2026-10-17，用户授权）。`master` 已上线，`default_branch=master`，
  GitHub 许可证识别为 **GPL-3.0**。**尚无 tag / Release**。

### 9.2.1 ✅ 推送前清理（2026-10-17 已完成）

1. ✅ **`Cargo.toml` 占位符已修**：`repository = "https://github.com/SyntaxEntity/Pomato"`。
2. ✅ **文档内个人化示例值与绝对家目录路径已清**：`README.md` · `docs/CONFIGURATION.md` ·
   `docs/BUILD-AND-RELEASE.md` · `docs/ROADMAP.md` · 本文件 —— 现在全仓库无 `/home/<user>/` 与个人目标名。
   验证方式：**用「树里必然不存在」的随机串**当搜索模式，再把自己的家目录用户名 `sed` 进去
   （不要把这两个值直接写进命令，否则命令会匹配到自己，产生假阳性）：

   ```bash
   SENTINEL="$(head -c8 /dev/urandom | base64 | tr -dc 'a-z' | head -c8)"   # 随机且不可能命中
   git grep -nE "$SENTINEL|/home/$(whoami)/" HEAD -- .    # 期望：0 命中
   ```

   > 含真实家目录路径的个人文件已移出仓库树（在 `.dsh/`，已 gitignore），
   > 所以上面的 `git grep`（只查提交树）必须为 0。
   >
   > 另注：`src/app.rs` 与 `src/config.rs` 中的「考研」是**通用示例值**（UI placeholder / 测试夹具），
   > 不是个人信息，刻意保留。
3. ✅ **Vibe-Coded 披露卡片已加**：`README.md` 顶部徽章 + 底部披露段（`logo=deepseek`，非 `robot`）。
4. ✅ **隐私审计已通过**（`--tree HEAD`）。曾因审计器把 SSH 地址误判为邮箱而临时改写远端表述；
   **该审计器 bug 已由 Atlas 修复**，远端地址已写回完整形式 `git@github.com:SyntaxEntity/Pomato.git`。
   **不要**用 `--no-verify` 绕过审计。

### 9.3 发版规则的补充

- tag 格式统一 `v<MAJOR>.<MINOR>.<PATCH>`，annotated tag。
- **发版必须配 GitHub Release**，且 Release 说明**必须包含「已知边界」**（未验证的平台 / 未跑通的路径要写明，不许暗示已验证）—— 0.1.12 的两项修复（全屏同步、日期弹窗不收起）**已经用户真机验收通过**（2026-10-20）；0.1.14 的窗口最小尺寸改动已通过真实运行验证（窗口最小 578×745），但**用户尚未安装确认**，发版时须如实标注。
- 版本号一致性：`Cargo.toml` / 系统已装包 / tag **三处核对**。

### 9.4 与项目既有约定的关系

本项目 §3 的 12 条硬性约束**全部继续有效**。Atlas 规范只做**增加**，不放松任何一条 ——
尤其：**不擅自 `git init`**、**不擅自打包发版**、**agent 不能 sudo**、**不在用户实时桌面盲启 GUI**。
