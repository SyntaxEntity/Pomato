# 已知问题

> 状态截至 0.1.12 代码（**未打包**）。优先级标注：高 / 中 / 低。
> 待办排序与「明确不做」的清单见 [`ROADMAP.md`](ROADMAP.md)。

---

## 1. 全屏状态不同步 —— ✅ 已修（0.1.12 代码，未验收）

**优先级：高（原 P0）**

### 原因
`App.fullscreen` 是自维护 bool，只在 `ToggleFullscreen` / `ExitFullscreen` 里翻转。用户用窗口管理器退出全屏（Super 键、拖拽、热角等）时 flag 不更新，导致：

- 全屏背景图 / 遮罩残留，或该出现时不出现（`apply_fullscreen_bg` 只看这个 flag）
- F11 方向判断错乱（以为在全屏其实不在，或反之）

### 修法（已实施）
**删掉镜像状态，而不是同步它**：移除 `App.fullscreen` 字段；新增 `Msg::FullscreenChanged(bool)`，由 `root.connect_fullscreened_notify` 发 `window.is_fullscreen()` 驱动 `apply_fullscreen_bg(fullscreened)`；F11 改为查 `window.is_fullscreen()` 决定方向，Escape 改为无条件 `unfullscreen()`（幂等）。进入/退出都不再手动写状态。

### ⚠️ 尚未做运行态验证
本环境是用户的**实时 Wayland 会话**（`DISPLAY=:0` / `wayland-0`），且：

- `/run/user/1000` 只读 → 新建 Wayland client 连不上
- 无 `Xvfb` / `xdotool` → 无法隔离起测试实例

因此没有在用户桌面上盲启 GUI。**已完成的验证**：

- ✅ `cargo test` 8/8 全绿
- ✅ debug / release 编译通过
- ✅ `cargo clippy` 无新增警告
- ✅ 断言检查：代码中已无 `self.fullscreen`；`FullscreenChanged` 仅 1 个生产者；`apply_fullscreen_bg` 仅 2 个调用点且均传真实窗口状态
- ✅ 确认 `window.is_fullscreen()` 与 `connect_fullscreened_notify()` 存在于 gtk4 0.11 绑定

**真机验收步骤（待用户执行）**：

1. 起 0.1.12 → 设好全屏背景 → F11 进全屏：背景图 + 遮罩应出现
2. 用 Super 键或拖拽等方式退出全屏（**不走** F11/Escape）：背景/遮罩应立刻消失，无残留
3. 再按 F11：应能正常进入全屏（方向判断不再错乱）
4. Escape 退出、标题栏全屏按钮进出，各一次

### 遗留点
进入全屏后的显隐完全依赖 notify，没有「先手动 apply 一次」的乐观兜底。若将来在某个合成器上出现背景延迟一拍，可在 `ToggleFullscreen` 里补一次乐观 apply。

---

## 2. 计时状态不持久化（中）

GSettings 只存「配置」，**不存会话**。重启或崩溃后：正在跑的倒计时、剩余秒数、`completed_work`（第几个番茄）全部清零。

⚠️ **做之前必须先问用户**。用户只确认过「设置要保」，计时会话是否需要续传**从未提过**。

**修法参考**：用 GSettings 或本地 state 文件存 `phase / status / remaining_secs / completed_work / last_tick`，启动时恢复。

---

## 3. 日期弹窗选完不自动收起 —— ✅ 已修（0.1.12 代码，未打包）

`goal_date_row` 的 `connect_day_selected` 原先只写设置 + 改按钮文案，**没有** `popover.popdown()`；反倒是「清除日期」会收起弹窗，体验割裂。

**修法（已实施）**：选中后经 `glib::timeout_add_local_once(150ms)` 延后一拍再 `popdown()`——`day_selected` 在点击的同一次事件处理中触发，立刻收起会与 `gtk::Calendar` 内部的按压处理抢时序。

**连带处理**：「清除日期」按钮内部会调 `select_day()`，那同样会触发 `day_selected`。用 `Rc<Cell<bool>>` 抑制标志让该路径跳过排程，避免重复 `popdown()`（幂等但语义混乱）。

**未做运行态验证**：无隔离 GUI 环境，时序行为需真机确认（点选一天后面板应立即消失、且不出现与日历动画打架的闪烁）。

---

## 4. 全屏仍显示 HeaderBar（中，视产品预期）

进全屏后标题栏还在，背景图被压在 ToolbarView content 下面，顶部不沉浸。若期望「全屏纯背景 + 计时」，需要全屏时隐藏 `header`（`adw::HeaderBar` 在 `add_top_bar`，可用 ToolbarView 的 top bar reveal 或直接 `set_visible(false)`）。

⚠️ **隐藏前必须确认退出全屏的路径仍在**（F11 / Escape），否则用户会困在全屏。
⚠️ **做之前先和用户确认是否要这个效果。**

---

## 5. CSS blur 边缘发虚 / 露底（低）

`filter: blur()` 作用在 `picture.bg-picture` 上，图片边缘会向外扩散出半透明边，在遮罩不透明度低时能看到发白/发灰的一圈。

**常见补偿**：给 Picture 加约 1.05 倍 scale，或用负 margin 裁掉边缘。遮罩色/不透明度已可调，不明显时可先不动。

---

## 6. 提示音很弱（低）

`notifier::beep()` 只是 `gdk::Display::beep()`（系统蜂鸣），不是可配置音效。设置里叫「提示音」但没有音量/音色选项。

---

## 7. 遗留文件易误导（低）

`data/icons/pomato.svg` 仍在树里，但**不是**当前图标源（当前是 Pillow 脚本生成的简笔番茄 PNG）。

**处置**：可删除，或在文件头加注释标明是遗留稿。**不要**用它去 regenerate 图标。

---

## 8. 代码小瑕疵（低）

- 倒数日 / 日期 UI 无自动化测试（只有 config / timer 单测）
- 偏好设置里的 SpinRow 显示原始浮点（如 `0.6000000000000001`），可改为 `%` 显示或改用 Scale
- `cargo clippy` 有 7 条既有警告（unnecessary cast / let-binding / collapsible if / manual_is_multiple_of）。
  均为既有技术债，**未获清理授权**，不要顺手清（会混进无关 diff）

> `app.rs` 的 `switch_row` 签名挤行问题、以及 `cargo fmt --check` 的差异，
> 已于 2026-10-20 由 `cargo fmt --all` 一并清零（裁定 D-14）。

---

## 9. 窗口拖得太小时控件溢出 —— ✅ 已修（0.1.14）

**现象**（用户 2026-10-20 实测）：窗口可以拖到很小，内容区的进度环 / 标签 / 按钮会跑到窗口外面。

**根因（两次尝试才定位）**：

内容区是一个 `gtk::Overlay`，`bg_picture`、`bg_scrim` 和内容 Box **三个全用 `add_overlay` 挂上去**，
于是 Overlay **没有主 child**。而 GTK4 的 `GtkOverlay` **只测量主 child，overlay 子件不参与尺寸计算** ——
它的 `measure()` 恒为 `0×0`，最小尺寸传播链在这里断掉，窗口下限退化成 HeaderBar 的 `360×200`。

> ⚠️ **踩坑记录**：第一版（0.1.13）只加了 `set_size_request: (420, 680)`，**无效**。
> 因为 size request 是加在那层 Box 上的，而 Box 的 `measure` 结果根本没被 Overlay 采纳。
> 实测证据（加临时探针打印逐层 `measure`）：
> ```
> GtkBox      min=578x699   ← size request 生效了
> GtkOverlay  min=0x0       ← 被归零
> AdwToolbarView min=206x46
> AdwApplicationWindow min=360x200   ← 窗口最小尺寸
> ```

**修法（已实施）**：让内容成为 Overlay 的 **主 child**（`set_child`），两张背景图继续做 overlay：

```rust
#[name = "content"]
#[wrap(Some)]
set_child = &gtk::Box { ... }      // 原为 add_overlay
```

修复后逐层实测：

```
GtkBox      min=578x699
GtkOverlay  min=578x699    ← 不再是 0x0
AdwToolbarView min=578x745
AdwApplicationWindow min=578x745   ← 窗口拖不到比这更小
```

**副产物**：内容真实最小尺寸是 **578×699**（受按钮行 530 + 左右 margin 48 约束），
比原默认窗口 560×720 还大，所以默认尺寸一并提到 **620×780**，避免启动时窗口被迫撑大。

**为什么不用 `set_geometry_hints`**：GTK4 已移除该 API（`libgtk-4.so.1` 里无此符号），
gtk4-rs 0.11 也未绑定。最小尺寸必须靠内容 `measure` 自然传播。

**验证状态**：窗口最小尺寸已在真实运行中实测为 **578×745**（临时探针，已删除）。
用户装 0.1.14 后可拖拽确认。

---

## 10. 窗口右边缘的深色细线 —— ✅ 已修（0.1.17，待验收）

**现象**（用户实测）：窗口最右边缘有一条 1-2px 的深色线，「拖动就没了」。

### 定位过程与实测数据

环境事实：

```
显示器逻辑缩放 1.25（GNOME fractional scaling）
GTK scale_factor = 2      ← 不是 1.25
合成器把 2x 缓冲缩小 0.625 倍显示
```

逐像素对比 Pomato 与 Glance（同为 GTK4 + libadwaita + Rust，且用户指出「其他应用没有」）：

```
Glance 右边缘: content(245,241,232) | 220, 208, 217, 222, 225, 230 …
Pomato 右边缘: content(250,250,251) | 180, 208, 217, 222, 225, 230 …
                                      ↑ 只有这里不同
```

两条衰减序列**完全一致**，唯一差异是紧贴内容的那一像素：Glance 220（浅）、Pomato 180（深）。

排除过程（均用临时真机测）：

- **不是布局留边**：实测 window=620 / content=620，宽度相等
- **不是 Overlay**：临时移除 Overlay 层，右边缘值不变
- **不是自定义 CSS**：完全禁用 style.css，值反而更深（167）
- **不是窗口尺寸奇偶**：扫过 W=600/604/608/616/624/632/640/700，右边缘恒为 180
- **不是 backdrop**：`is_active` 与 `is-focus` 均为 true
- **与背景有关**：不同窗口尺寸/位置下测得 180~243 不等 —— 该像素是**半透明阴影**，与背后内容混合，在深色壁纸上就显得像黑线

**根因**：Adwaita 的 CSD 阴影最内圈是半透明的，窗口背景没有覆盖到这一像素，
于是阴影与桌面直接混合；深色壁纸下呈现为可见的深色边。

### 修法演进

**0.1.17 尝试**：加 `border: 1px solid @window_bg_color` —— 只能把 180 提到 191，
用户实测仍有可见暗线。**未根治**。

**0.1.18 根因确认**：从 libadwaita 的 gresource 里取到 Adwaita 的窗口阴影定义：

```css
box-shadow: 0 0 0 1px color-mix(in srgb, var(--shade-color) 75%, transparent),
            0 2px 8px 2px var(--shade-color), 0 3px 20px 10px RGB(0 0 6/5%), …
```

最内圈那 1px 是 **shade-color 的 75%（约 19% 黑）**——在浅色内容与深色壁纸交界处
就是一条明显的深色细线。这是 Adwaita 默认阴影的固有表现。

**最终修法**：用更轻的自定义阴影替换默认值，保留层次感、去掉深色内圈：

```css
window.background {
  box-shadow: 0 0 0 1px RGB(0 0 6 / 4%), 0 2px 6px RGB(0 0 6 / 8%);
}
```

**实测效果**：边缘最内像素 **180 → 218**（与 Glance 的 220 一致），衰减单调、无凹陷。
`box-shadow: none` 也能消除（边缘变成纯背景色），但窗口会失去立体感，故采用轻阴影方案。

**验证方式**：本轮起可用 XDG portal 截图（`gdbus call … org.freedesktop.portal.Screenshot.Screenshot`，
`interactive=false` 即可，文件落在 `~/Pictures/Screenshot-N.png`），据此做了上面的像素级对比。

**待用户确认**：装上 0.1.17 后右边缘的深色线是否消失。
