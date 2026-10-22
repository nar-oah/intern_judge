# Intern Judge

Bevy 0.20 + Avian 0.8 的简单平台关卡示例。运行：`cargo run`。

- A / D：移动；空格：跳跃；E：与碰撞箱重叠的建筑或切换点交互。
- 住宅为绿色、商店为橙色、工厂为紫色，切换点为青色；建筑交互输出到日志。
- 与切换点交互后，点击 Bevy 原生 UI 中的目的地按钮；Cancel 或 Esc 关闭界面。
  菜单通过 Bevy 原生 `GamePhase` 状态在 `Exploring` 和 `ChoosingDestination`
  之间切换；仅探索状态允许移动、跳跃和 E 键交互。打开菜单时清零玩家水平速度，
  物理模拟持续运行，玩家仍会受到重力和碰撞影响。

初始路段为 `example_road`，走到右侧切换点可选择 `market_road` 或法院
`RoadId("court")`。选择 `market_road` 后，从新路段右侧切换点可前往法院。
路段之间单向前进；每次切换删除旧路段根实体及全部子实体，只加载一个路段，
保留玩家和摄像机，玩家从最左侧 `(-460, -130)` 出生并清零速度。
法院通过 `court_scene()` 定义为普通路段，根实体标识为 `RoadId("court")`，
目前仅有一个地面平台；进入后可正常移动和跳跃。
UI 使用英文标签和 Bevy 默认字体，无需额外字体资源。

关卡场景按平台、共用建筑、各类建筑、切换点和路段组合拆分到
`src/level/scene/`，由 `scene.rs` 组织模块。场景使用 Bevy 0.20 的 `bsn!`、
`@scene(...)` 和 `Children [ ... -- ... ]` 组合。

`platform_scene(position, size)` 的尺寸写入 `Platform` 组件，并同时用于 Sprite
和 Collider，方便以后替换为九切片或平铺贴图。三种建筑各自有场景函数和
`BuildingType` 组件，共用建筑占位场景。`SwitchPoint` 仅保存
`destinations: &'static [RoadId]`，由每个切换点自行配置可前往的路段，不再包含 `id`。

新增路段时，按 `example_road_scene` 的形式定义返回 `impl Scene` 的函数，
为根实体分配独立、固定的 `RoadId`，在 `src/level/scene.rs` 的 `spawn_road`
入口中登记一次 ID 与场景函数的对应关系，再将该 ID 加入前一路段切换点的
`destinations`。`navigation.rs` 统一完成目标验证、旧路段销毁、新场景生成和
玩家运动状态重置，无效目的地不会销毁当前路段。
子实体的位置相对于根实体；平台用尺寸指定形状，根实体保持单位缩放。

建筑整体和切换点使用与占位外观同尺寸的 `Collider + Sensor + CollidingEntities`
检测重叠，允许玩家穿过。统一交互逻辑位于 `src/level/interaction.rs`，
每次新按下 E 向所有与该玩家重叠的可交互目标发送 `InteractRequested` EntityEvent，
事件包含目标和玩家实体。各类建筑和切换点通过 BSN 的 `on(...)` 绑定自己的
Observer；路段切换由 `navigation.rs` 处理，目的地选择界面位于 `ui.rs`。
暂不包含路线图、路网生成、路段解锁、建筑经济效果或实际判案逻辑。

验证：`cargo test`、`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`。
