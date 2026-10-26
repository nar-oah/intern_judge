# Intern Judge

Bevy 0.20 + Avian 0.8 的简单平台关卡示例。运行：`cargo run`。

- A / D：移动；空格：跳跃；E：与碰撞箱重叠的建筑或切换点交互。
- 按住左 Shift 并水平移动时以 1.5 倍速度加速，每秒消耗 5 点体力。
  体力初始值和上限为 100，不会自然恢复；耗尽后仍可正常移动和跳跃。
  松开左 Shift 或停止水平移动时不消耗体力。
- 住宅为绿色、商店为橙色、工厂为紫色，切换点为青色；建筑交互输出到日志。
- 与切换点交互后，进入全屏、不透明的目的地选择 UI，点击有效目的地按钮继续。
  UI 是独立的 BSN 场景；进入时卸载当前 Road 及 Player，不提供 Esc 或 Cancel。

初始路段为 `example_road`，走到右侧切换点可选择 `market_road` 或法院
`RoadId("court")`。选择 `market_road` 后，从新路段右侧切换点可前往法院。
路段之间单向前进，完整流程为 Road → 目的地选择 UI → 新 Road。
Road 和 UI 的根实体统一由 `SceneRoot` 标记；每次切换删除旧场景根实体及全部
子实体，再生成新场景。Camera 和右上角通勤 HUD 等全局实体始终保留。
每个 Road 通过 `player_scene(Vec2::new(-460.0, -130.0))` 定义自己的 Player
子场景和出生位置；进入新 Road 时生成新的 Player，移动、跳跃和物理逻辑照常运行。
Player 不存在时，移动和交互系统自然跳过。
法院通过 `court_scene()` 定义为普通路段，根实体标识为 `RoadId("court")`，
目前仅有一个地面平台；进入后可正常移动和跳跃。
UI 使用英文标签和 Bevy 默认字体，无需额外字体资源。

`src/game.rs` 的 `GamePlugin` 管理全局 `CommuteTime`、`Money`、`Stamina` 和
`CommuteState`，默认通勤时限为 300 秒，已用时间和金钱均为 0。
体力保存在全局 `Stamina` 资源中，切换路段或重新生成 Player 不会重置。
计时系统只在 `Running` 状态执行；进入目的地选择 UI 时请求 `Paused`，
选择普通 Road 后恢复 `Running`，进入 Court 后保持 `Paused`。
状态请求由 Bevy 在下一次状态切换调度中应用，不影响物理和玩家移动。
超时后已用时间继续增长，`remaining()` 直接返回时限减去已用时间，可为负数。
HUD 独立于 `SceneRoot`，实时显示剩余秒数、金钱和可视化体力条，
在全屏目的地 UI 上也保持可见。

关卡场景按平台、共用建筑、各类建筑、切换点和路段组合拆分到
`src/level/scene/`，由 `scene.rs` 组织模块。场景使用 Bevy 0.20 的 `bsn!`、
`@scene(...)` 和 `Children [ ... -- ... ]` 组合。

`platform_scene(position, size)` 的尺寸写入 `Platform` 组件，并同时用于 Sprite
和 Collider，方便以后替换为九切片或平铺贴图。三种建筑各自有场景函数和
`BuildingType` 组件，共用建筑占位场景。`SwitchPoint` 仅保存
`destinations: &'static [RoadId]`，由每个切换点自行配置可前往的路段，不再包含 `id`。

新增路段时，按 `example_road_scene` 的形式定义返回 `impl Scene` 的函数，
为根实体分配独立、固定的 `RoadId`，在 `src/level/scene.rs` 的 `road_scene`
入口中登记一次 ID 与场景函数的对应关系，再将该 ID 加入前一路段切换点的
`destinations`。`navigation.rs` 负责目的地验证、场景切换及通勤状态请求；无效目的地、重复或
已失效的交互不会卸载当前场景或重复生成场景。
子实体的位置相对于根实体；平台用尺寸指定形状，根实体保持单位缩放。

建筑整体和切换点使用与占位外观同尺寸的 `Collider + Sensor + CollidingEntities`
检测重叠，允许玩家穿过。统一交互逻辑位于 `src/level/interaction.rs`，
每次新按下 E 向所有与该玩家重叠的可交互目标发送 `InteractRequested` EntityEvent，
事件包含目标和玩家实体。各类建筑和切换点通过 BSN 的 `on(...)` 绑定自己的
Observer；路段切换由 `navigation.rs` 处理，目的地选择界面位于 `ui.rs`。
暂不包含路线图、路网生成、路段解锁、建筑经济效果或实际判案逻辑。

验证：`cargo test`、`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`。
