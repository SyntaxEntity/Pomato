# 配置项（GSettings）

- **schema**：`io.github.pomato.Pomato`
- **路径**：`/io/github/pomato/`
- **定义文件**：`data/io.github.pomato.Pomato.gschema.xml`
- 存储位置：**dconf**（不是文件配置）。重启、升级 RPM 均不丢失。

---

## 键位表

| 键 | 类型 | 取值 | 默认 | 说明 |
|---|---|---|---|---|
| `work-minutes` | u | 1–90 | 25 | 专注时长（分钟） |
| `short-break-minutes` | u | 1–90 | 5 | 短休时长（分钟） |
| `long-break-minutes` | u | 1–90 | 15 | 长休时长（分钟） |
| `long-break-interval` | u | 1–12 | 4 | 每完成几个专注进入长休 |
| `auto-start-next` | b | — | false | 自动开始下一阶段 |
| `sound-enabled` | b | — | true | 阶段结束提示音 |
| `notifications-enabled` | b | — | true | 阶段结束桌面通知 |
| `close-to-tray` | b | — | true | 关闭窗口时收进托盘 |
| `minimize-to-tray` | b | — | true | 最小化时收进托盘 |
| `show-tray-icon` | b | — | true | 显示托盘图标 |
| `goal-name` | s | — | `''` | 总目标名称 |
| `goal-date` | s | `YYYY-MM-DD` | `''` | 总目标日期 |
| `fullscreen-bg-path` | s | 文件路径 | `''` | 全屏背景图（0.1.10+）；空 = 全屏也用原生背景 |
| `fullscreen-bg-dim` | d | 0–1 | 0.45 | 遮罩不透明度 |
| `fullscreen-bg-blur` | d | 0–64 | 12.0 | 背景模糊半径（px） |
| `fullscreen-bg-color` | s | CSS 颜色 | `'black'` | 遮罩颜色（见下方 UI 说明） |

> 类型记号：`u` = uint32，`b` = bool，`s` = string，`d` = double。
>
> `fullscreen-bg-color` 在偏好设置里有三种改法：**「黑」/「白」一键按钮**（写 `black` / `white`）、**`ColorDialogButton` 色板**（自定义任意颜色，写入 CSS 颜色串）、以及直接 `gsettings set`。也可以填任意合法的 CSS 颜色，例如 `rgba(0,0,0,0.5)`。

---

## 命令行读写

```bash
# 查看全部当前值
dconf dump /io/github/pomato/

# 单个键
gsettings get io.github.pomato.Pomato work-minutes
gsettings set io.github.pomato.Pomato work-minutes 50

# 重置单个键
gsettings reset io.github.pomato.Pomato work-minutes
```

---

## 备份与恢复

仓库内 `dist/` 下有两个**角色不同**的文件，别混：

| 文件 | 角色 |
|---|---|
| `dist/pomato-settings-backup.ini` | **备份数据**（一份 dconf dump 快照），不是脚本 |
| `dist/restore-pomato-settings.sh` | **恢复脚本**，内部执行 `dconf load` |

```bash
# 恢复（用仓库里的备份数据）
dist/restore-pomato-settings.sh

# 手动备份（仓库没有备份脚本，自己 dump 即可）
dconf dump /io/github/pomato/ > my-pomato.ini

# 手动恢复
dconf load /io/github/pomato/ < my-pomato.ini
```

换机迁移：把 `dist/pomato-settings-backup.ini` 拷过去，然后 `dconf load /io/github/pomato/ < pomato-settings-backup.ini` 即可。

---

## 开发期注意事项

- `build.rs` 会把 `data/io.github.pomato.Pomato.gschema.xml` 编译到 `OUT_DIR`，开发期通过环境变量 `POMATO_SCHEMA_DIR` 定位。
- 安装后则走系统 schema（`/usr/share/glib-2.0/schemas/`），RPM 的 `post_install_script` 会自动跑 `glib-compile-schemas`。
- ⚠️ **改了 gschema 必须重新 build 才生效**，否则读到的是旧的编译产物。

---

## 示例配置（占位值，可直接导入）

`dist/pomato-settings.example.ini` 是一份**脱敏模板**，含全部键的占位值，可直接导入试用：

```bash
dconf load /io/github/pomato/ < dist/pomato-settings.example.ini
```

> ⚠️ `dist/pomato-settings-backup.ini` 与 `dist/restore-pomato-settings.sh` 是**开发者本机的个人迁移工具**，
> 含真实路径与个人目标，已被 `.gitignore` 排除，**不随仓库分发**。

一个可参考的「专注场景」配置样例（用于复现较长专注时长下的界面表现）：

| 键 | 示例值 |
|---|---|
| 专注 / 短休 / 长休 | 50 / 10 / 15 分钟 |
| 长休间隔 | 4 |
| 目标名称 | `项目截止` |
| 目标日期 | `2026-12-31` |
| 全屏背景 | 任意本地 PNG（`fullscreen-bg-path`） |
| 遮罩颜色 / 不透明度 | 白色 / `0.6` |
| 模糊 | `0` |

> 注：偏好设置里的不透明度 SpinRow 会显示原始浮点（如 `0.6000000000000001`），是已知瑕疵，见 [`KNOWN-ISSUES.md`](KNOWN-ISSUES.md)。
