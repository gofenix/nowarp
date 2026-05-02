# Nowarp 定制指南

Nowarp 的目标不是保留 Warp 的完整产品面，而是做一个更直接的终端工作台。当前改动已经把体验收敛到「打开就是终端、默认使用侧边栏 tab、尽量不打扰用户」；后续继续改时，优先围绕这个方向判断取舍。

## 产品原则

1. 侧边栏 tab 是主导航，不是实验入口。
   新窗口、新会话、恢复窗口时都应默认进入 vertical tabs 体验。水平 tab 可以作为兼容路径存在，但不应再作为默认心智。

2. 终端优先，账号和增长流程靠后。
   用户应该能在未登录、未 onboarding、未配置 AI 的情况下直接进入可用终端。登录、团队、账单、推荐、feature promo、launch modal 这类流程不能挡住 terminal-first 路径。

3. 不用的 AI feature 最终应该移除，不只是用开关隐藏。
   现在很多 AI/agent/onboarding surface 通过 `suppress_automatic_*`、settings nav 过滤、feature flag 判断等方式隐藏。这是为了快速压低干扰和降低 merge 风险，不应长期成为产品结构。后续要按入口、状态、后台订阅、设置字段、测试逐步删除。

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

## 当前仍是“隐藏”的部分

这些地方现在还不是彻底删除，只是把入口或自动触发压掉：

- `app/src/onboarding_suppression.rs` 统一返回 `true`。
- Settings 的 AI、MCP、Billing、Teams、Code、Privacy、Referrals、WarpDrive、Warpify 等页面仍有 backing implementation，只是不在侧栏里出现。
- AI assistant panel、agent settings、voice input、billing usage、teams workspace 等代码仍被编译和部分订阅，只是很多自动弹窗和提示不再触发。
- 一些 setting 字段仍会存在，用于兼容旧配置、旧快照和上游代码路径。

这部分要当作过渡状态：先保证用户看不到，再逐步让代码也不存在。

## 后续移除 AI feature 的顺序

1. 先删可见入口。
   从 toolbar、settings sidebar、command palette、快捷键、菜单、tooltip、toast、modal、welcome/warm welcome 入口开始。标准是用户没有任何自然路径打开不用的 AI feature。

2. 再断自动触发和后台订阅。
   删除或收敛 `AISettings` change subscription、one-time modal 检查、onboarding flow、warm welcome、voice/code feature popup、agent notification 这类自动行为。

3. 然后删状态和设置字段。
   确认没有恢复、迁移、同步、测试依赖后，再删除对应 setting、snapshot 字段、private preference key、telemetry event 和默认值。

4. 最后删 backing view/model/crate。
   页面、panel、model、client、crate 只有在没有入口、没有订阅、没有持久化依赖后再删除。每次删除保持小步，方便和上游 master 合并。

## 改动检查清单

改一个入口或 feature 时，先问：

1. 它是否服务于 terminal 或侧边栏 tab？
2. 未登录用户是否需要它才能使用终端？
3. 它是否属于 AI/agent/growth/team/billing/referral/onboarding？
4. 如果隐藏入口，旧快照、搜索、快捷键、菜单是否还能打开它？
5. 是否有测试能证明默认体验仍然是 terminal-first 和 sidebar-tabs-first？

如果答案指向“不是终端核心能力”，默认策略是从用户可见面移除；只有为了兼容旧状态或降低上游合并风险时，才短期保留开关式隐藏。
