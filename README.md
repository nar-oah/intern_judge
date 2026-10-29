# Intern Judge

Bevy 0.20 + Avian 0.8 的简单平台关卡示例。运行：`cargo run`。以下数值为默认配置。

- A / D：移动；空格：跳跃；E：与碰撞箱重叠的建筑或切换点交互。
- 按住左 Shift 并水平移动时以 1.5 倍速度加速，每秒消耗 5 点体力。
  体力初始值和上限为 100，不会自然恢复；耗尽后仍可正常移动和跳跃。
  松开左 Shift 或停止水平移动时不消耗体力。
- 金钱初始为 100 元。便利店为绿色，Small 类型花费 10 元恢复 30 点体力，
  Large 类型花费 20 元恢复 60 点体力，最多恢复至 100 点；满体力或余额不足时不交易。
- 工具店为橙色，花费 20 元购买安全帽；已有安全帽或余额不足时不交易。
- 彩票店为紫色，每次花费 10 元，10% 概率获得 50 元、20% 概率获得 10 元，
  其余无奖励；余额不足时不交易。
- 猫咪在右侧高台上，需要跳跃靠近后按 E 救援，获得 20 元并删除该猫咪场景实例。
  切换点为青色。
- 与切换点交互后，进入全屏、不透明的路网地图，点击绿色边框的可达路段继续。
  UI 是独立的 BSN 场景；进入时卸载当前 Road 及 Player，不提供 Esc 或 Cancel。

地图显示完整路网，法院为金色、当前路段为蓝色，可前往路段有绿色边框，
其余节点仅展示。连线直接连接节点中心，地图按窗口短边等比缩放。
`src/level/map.rs` 只读取 `RoadGraph`，每次进入 `Playing` 时根据本局路网重新生成 `MapLayout`：
法院位于原点，将路网视为无向图做 BFS，同层路段按稳定的 ID 顺序均匀分布在圆环上；
正常路网假设与法院连通。布局只负责坐标，实际通行仍由有向路网和导航校验决定。
`ui.rs` 使用原生 UI / BSN 绘制节点和连线，并复用原有目的地选择逻辑。

游戏从逻辑出生点 `RoadId::Start` 直接打开地图，初始可选择 `RoadId::ExampleRoad`。
Start 仅用于路网和地图展示，没有关卡场景，不生成玩家或物理实体，也不作为经过路段。
进入 ExampleRoad 后，走到右侧切换点按 E 可选择 `RoadId::MarketRoad`
或法院 `RoadId::Court`。选择 MarketRoad 后，从新路段右侧切换点可前往法院。
路段之间单向前进，完整流程为 Start → 地图 UI → Road → 地图 UI → Road → Court。
Road 和 UI 的根实体统一由 `SceneRoot` 标记；每次切换删除旧场景根实体及全部
子实体，再生成新场景。Camera 和右上角通勤 HUD 等全局实体始终保留。
每个 Road 通过 `player_scene(Vec2::new(-460.0, -130.0))` 定义自己的 Player
子场景和出生位置；进入新 Road 时生成新的 Player，移动、跳跃和物理逻辑照常运行。
Player 不存在时，移动和交互系统自然跳过。
法院通过 `court_scene()` 定义为普通路段，根实体标识为 `RoadId::Court`，
目前仅有一个地面平台；进入后可正常移动和跳跃。
UI 使用英文标签和 Bevy 默认字体，无需额外字体资源。

静态配置位于 `assets/`，修改对应 RON 文件后重新运行即可生效：

- `game.settings.ron`：`commute_limit` 为通勤时限，`initial_money` 为初始金钱，
  `multiplier` 为冲刺速度倍率，`per_second` 为每秒体力消耗。
- `roads.graph.ron`：`roads` 按 `RoadId` 枚举值配置下一路段数组；空数组表示没有目的地。
  `Start` 初始连接 `ExampleRoad`，不能作为可进入的目的地。
- `buildings.values.ron`：`cat_reward` 为统一猫咪奖励；`lottery` 配置彩票 `price`
  及 `prizes`，每项使用 `reward` 和 `weight` 定义奖励与相对权重；
  `convenience_stores` 按 Small、Large 类型配置 `price` 和 `stamina_recovery`。

`src/config.rs` 集中定义配置类型与 `ConfigPlugin` 加载逻辑，使用
`bevy_asset_loader` 加载、`bevy_common_assets` 反序列化 RON。启动时进入
`GameState::Loading`，全部配置加载完成并从静态 Asset 转为只读 Resource 后，
进入 `GameState::Playing`，再初始化游戏、HUD 和关卡场景。系统直接读取
`GameSettings`、`RoadGraph`、`BuildingValues`，无需反复查询 `Assets<T>`。

`src/game.rs` 的 `GamePlugin` 管理全局 `CommuteTime`、`Money`、`Stamina`、
`HasHelmet` 和 `CommuteState`。固定参数直接读取配置，`Money` 和
`CommuteTime.elapsed` 仍是运行时资源；金钱按 `initial_money` 初始化，已用时间
初始为 0，初始没有安全帽。`Stamina::MAX` 保持硬编码为 100。
体力、安全帽和金钱保存在全局资源中，切换路段或重新生成 Player 不会重置。
计时系统只在 `Running` 状态执行；初始地图为 `Paused`，选择首个实际 Road 后才开始计时。
后续进入目的地选择 UI 时请求 `Paused`，选择普通 Road 后恢复 `Running`，进入 Court 后保持 `Paused`。
状态请求由 Bevy 在下一次状态切换调度中应用，不影响物理和玩家移动。
超时后已用时间继续增长，`remaining(limit)` 接收配置中的通勤时限，返回时限减去
已用时间，可为负数。
HUD 独立于 `SceneRoot`，实时显示剩余秒数、金钱和可视化体力条，
在全屏目的地 UI 上也保持可见。

关卡场景按平台、共用建筑、各类建筑、切换点和路段组合拆分到
`src/level/scene/`，由 `scene.rs` 组织模块。场景使用 Bevy 0.20 的 `bsn!`、
`@scene(...)` 和 `Children [ ... -- ... ]` 组合。

`platform_scene(position, size)` 的尺寸写入 `Platform` 组件，并同时用于 Sprite
和 Collider，方便以后替换为九切片或平铺贴图。便利店、工具店、彩票店和猫咪
各自有场景函数和 `BuildingType` 组件，共用 `building_scene` 占位场景。
便利店实体通过 `ConvenienceKind` 指定 Small 或 Large，其余同类建筑共享配置。
工具店保持固定花费 20 元购买安全帽，不加入配置。`RoadId` 是支持反序列化的枚举，
`SwitchPoint` 是标记组件，由 `navigation.rs` 根据所在路段的 `RoadId` 查询
`RoadGraph` 中的可选目的地。

新增路段时，按 `example_road_scene` 的形式定义返回 `impl Scene` 的函数，
为 `RoadId` 增加枚举变体并分配给根实体，在 `src/level/scene.rs` 的 `road_scene`
入口中登记 ID 与场景函数的对应关系，再编辑 `assets/roads.graph.ron`，添加新路段
及前一路段可前往的目的地。`navigation.rs` 负责目的地验证、场景切换及通勤状态请求；无效目的地、重复或
已失效的交互不会卸载当前场景或重复生成场景。
子实体的位置相对于根实体；平台用尺寸指定形状，根实体保持单位缩放。

建筑整体和切换点使用与占位外观同尺寸的 `Collider + Sensor + CollidingEntities`
检测重叠，允许玩家穿过。统一交互逻辑位于 `src/level/interaction.rs`，
每次新按下 E 向与玩家重叠的可交互目标发送一次 `InteractRequested` EntityEvent，
事件包含目标和玩家实体，不处理多个交互区域重叠的情况。各类建筑和切换点通过 BSN 的 `on(...)` 绑定自己的
Observer；路段切换由 `navigation.rs` 处理，目的地选择界面位于 `ui.rs`。
暂不包含随机路网生成、路段解锁、周末限制、每日猫咪刷新、跨场景救援状态保存、
公交站或实际判案逻辑。

验证：`cargo test`、`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`。
