# Nowarp 定制指南

Nowarp 的目标不是保留 Warp 的完整产品面，而是做一个更直接的终端工作台。当前改动已经把体验收敛到「打开就是终端、默认使用侧边栏 tab、尽量不打扰用户」；后续继续改时，优先围绕这个方向判断取舍。

## 产品原则

1. 侧边栏 tab 是主导航，不是实验入口。
   新窗口、新会话、恢复窗口时都应默认进入 vertical tabs 体验。水平 tab 可以作为兼容路径存在，但不应再作为默认心智。

2. 终端优先，账号和增长流程靠后。
   用户应该能在未登录、未 onboarding、未配置 AI 的情况下直接进入可用终端。登录、团队、账单、推荐、feature promo、launch modal 这类流程不能挡住 terminal-first 路径。

3. **只做开关隐藏，不做代码删除**。
   AI/agent/onboarding/account/billing/team/referral 等不在 nowarp 产品面内的能力，一律通过 `suppress_automatic_*`、settings nav 过滤、feature flag 判断、OR 短路等开关手段**保留上游代码**地隐藏，不删除、不重写上游实现。理由是 nowarp 需要持续从上游 master 合并，删/改上游逻辑会反复制造冲突。
   "开关做法"的形态：新增一个 nowarp-only 的 `pub(crate) fn`（如 `bypass_auth_for_custom_endpoint`、`suppress_automatic_*`），恒返回 nowarp 偏置的值，在上游函数或调用点用 `||` / `if` 引入短路；merge 上游时只需把这一处短路删掉，上游逻辑原样保留。

4. 可见入口比后台能力更重要。
   对用户不可见的功能不能通过恢复快照、搜索、快捷键、菜单、tooltip、warm welcome、one-time modal 等路径重新出现。隐藏入口时要同时处理恢复路径和默认落点。

5. 每个定制都要有测试固定产品意图。
   默认值、恢复行为、settings nav、匿名状态入口、onboarding suppression 这类行为都需要小测试兜住，避免上游合并后悄悄回到 Warp 默认体验。

## 已完成更新

### 侧边栏 tab 默认化

Commit: `0e8224b Enable vertical tabs by default`

- `appearance.vertical_tabs.enabled` 默认改为 `true`。
- 恢复窗口时默认打开 vertical tabs panel。
- vertical tabs 的行粒度默认从 `Panes` 改为 `Tabs`。
- 补了默认值和恢复行为测试，确保新窗口和恢复窗口都维持侧边栏 tab 心智。

### 终端优先启动

Commit: `1e659b5 Customize Warp terminal experience`

- 新增 `onboarding_suppression`，统一控制自动 onboarding、launch modal、新功能提示是否出现。
- 未登录启动时优先进入 terminal，不再默认弹出 pre-login onboarding。
- 登录后如果服务器还认为用户未 onboarded，会静默标记完成，而不是再打开 onboarding slides。
- 欢迎 tab、welcome tips、AI warm welcome、voice/code/agent 新功能提示都会被 suppress。
- HOA/Oz/OpenWarp launch modal 会被标记为已检查或已完成，不再自动弹出。

### Settings 变成终端相关入口

Commit: `1e659b5 Customize Warp terminal experience`

- Settings 默认页改为 `Appearance`，但不改 `SettingsSection::default()`，减少和上游默认枚举的耦合。
- Settings sidebar 只显示：
  - `Appearance`
  - `Features`
  - `Keyboard shortcuts`
- 被隐藏的 settings section 在恢复快照时会 normalize 到 `Appearance`，避免旧窗口恢复到 AI、Billing、Teams 等页面。
- 匿名或未登录状态下，右上角 avatar slot 改为设置按钮；已登录用户仍保留用户菜单。

### 本地运行隔离

Commit: `1e659b5 Customize Warp terminal experience`

- `script/run` 支持 `--data-profile`、`--clean-data-profile`、`--fresh-data-profile`。
- profile 只支持 debug/direct execution，不支持 release build 或 macOS `--open_with_launchd`。
- 这个能力用于快速验证不同本地状态，不应和产品默认行为混在一起。

### 被动建议接自定义 AI 端点（不需登录）

Commit: pending

沿用"开关"做法：在 `onboarding_suppression.rs` 加一个 nowarp-only 开关函数 `bypass_auth_for_custom_endpoint()`，在 `is_custom_endpoint_enabled` 的上游检查上做 OR 短路，让被动 prompt 建议走用户自己配的 OpenAI 兼容端点，不要求 Warp 登录。

- 上游 `is_active_ai_enabled` 链路、`is_custom_endpoint_enabled` 的其它 `&&` 子句、调用点全部保留。merge 上游时只要把 `|| bypass_auth_for_custom_endpoint()` 删掉即可。
- 只影响被动建议这一条路径（`passive_suggestions/maa.rs:540` 和 `legacy.rs:295`）。其它 AI 能力（autosuggest、code suggestions、natural language detection、agent mode 等）仍走 `is_active_ai_enabled`，未登录全 false，保持"静默其它"。
- 用户在 `agents.warp_agent.active_ai` 下配 `custom_endpoint_enabled` / `custom_endpoint_base_url` / `custom_endpoint_model` / `custom_endpoint_api_key` 四个字段，端点协议见 `app/src/ai/custom_endpoint.rs`。
- `app/src/ai/blocklist/passive_suggestions/maa.rs:188` 会把端点 HTTP 错误静默吞掉，调试时临时加 `tracing::warn!` 能看到 `CustomEndpointError`。

## 持续被开关隐藏的部分

按原则 3，以下 surface 在 nowarp 中**长期**只通过开关隐藏，上游代码完整保留（这是终态，不是过渡）：

- `app/src/onboarding_suppression.rs` 统一返回 `true`。
- Settings 的 AI、MCP、Billing、Teams、Code、Privacy、Referrals、WarpDrive、Warpify 等页面仍有 backing implementation，只是不在侧栏里出现。
- AI assistant panel、agent settings、voice input、billing usage、teams workspace 等代码仍被编译和部分订阅，只是很多自动弹窗和提示不再触发。
- 一些 setting 字段仍会存在，用于兼容旧配置、旧快照和上游代码路径。
- 被动 prompt 建议的"登录要求"由 `bypass_auth_for_custom_endpoint` 开关绕掉（见下文"已完成更新"）。

**不要**把"隐藏入口"当成"将来再删代码"的过渡阶段。任何"后续清理上游实现"的工作都不在 nowarp 范围。

## merge 上游时的开关维护

新加一个 nowarp 开关时，按下面的形态落地，方便后续 merge：

1. 在 `app/src/onboarding_suppression.rs`（或同类 nowarp-only 模块）新增一个 `pub(crate) fn xxx_yyy() -> bool`，恒返回 nowarp 偏置值，并写 doc comment 说明意图。
2. 在上游函数或调用点用 `|| xxx_yyy()` / `if xxx_yyy() { ... }` 引入短路。**不要**改写上游已有的判断顺序、参数、签名。
3. 在 `NOWARP_GUIDE.md` 的"已完成更新"加一节记录这个开关、影响面、merge 时如何回退。
4. 如果是用户可见行为，配套在 `app/src/settings/ai_tests.rs`（或对应测试文件）加一个回归测试，断言开关开启时 nowarp 行为生效，防止 merge 上游后被悄悄改回去。

merge 上游时如果上游改动了被开关短路的那段逻辑：

- 只在开关调用点处出现冲突 → 直接接受上游改动，开关代码原样保留。
- 上游重写了被绕过的整个函数 → 按"开关做法"重新选一个稳定的 nowarp-only 短路点，迁移开关。
- 上游把对应能力**整个删了**（极少发生）→ 可以考虑把对应的 nowarp 开关和短路点一起删掉，但仍要先确认没有其他被旁路的能力依赖它。

## 改动检查清单

改一个入口或 feature 时，先问：

1. 它是否服务于 terminal 或侧边栏 tab？
2. 未登录用户是否需要它才能使用终端？
3. 它是否属于 AI/agent/growth/team/billing/referral/onboarding？
4. 如果隐藏入口，旧快照、搜索、快捷键、菜单是否还能打开它？
5. 是否有测试能证明默认体验仍然是 terminal-first 和 sidebar-tabs-first？

如果答案指向“不是终端核心能力”，默认策略是从用户可见面移除；只有为了兼容旧状态或降低上游合并风险时，才短期保留开关式隐藏。
