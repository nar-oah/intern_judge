# Intern Judge

Bevy 0.20 + Avian 0.8 的简单平台关卡示例。运行：`cargo run`。以下数值为默认配置。
启动流程为 Loading → MainMenu → Playing；配置加载后显示标题
“Intern Judge”和“Start Game”按钮，点击按钮开始本局游戏。

- A / D：移动；空格：跳跃；E：与碰撞箱重叠的建筑或切换点交互。
- 按住左 Shift 并水平移动时以 1.5 倍速度加速，每秒消耗 5 点体力。
  体力初始值和上限为 100，不会自然恢复；耗尽后仍可正常移动和跳跃。
  松开左 Shift 或停止水平移动时不消耗体力。
- 金钱初始为 100 元。以下交易和救援仅在 `CommuteState::Running` 时执行；
  `Paused` 时与商店或猫咪交互只打开信息对话及选项，不购买、发放奖励或救援猫咪。
  便利店为绿色，Small 类型花费 10 元恢复 30 点体力，
  Large 类型花费 20 元恢复 60 点体力，最多恢复至 100 点；满体力或余额不足时不交易。
- 工具店为橙色，花费 20 元购买安全帽；已有安全帽或余额不足时不交易。
- 有罪判决为 ExampleRoad 添加高空抛物效果。效果生效后，每次进入时在该路段配置的候选横坐标中随机选择一个，从上方生成
  一个彩色坠落方块。坠落物碰撞任何实体后消失；命中玩家时造成 2 秒眩晕，
  清零水平速度，保留垂直速度和物理模拟，结束后恢复移动操作。
  已有安全帽时消耗安全帽，免疫这次眩晕。
- 无罪判决为 MarketRoad 添加施工绕道效果，在两个商店之间生成醒目的施工路障，阻挡通行，初始跳跃能力即可越过。
  路障持续存在至离开路段，再次进入时按已保存的效果重新生成。
- 彩票店为紫色，每次花费 10 元，10% 概率获得 50 元、20% 概率获得 10 元，
  其余无奖励；余额不足时不交易。
- 猫咪在右侧高台上，需要跳跃靠近后按 E 救援，获得 20 元并删除该猫咪场景实例。
  切换点为青色。
- `Running` 时与切换点交互进入全屏、不透明的路网地图，点击绿色边框的可达路段继续；
  `Paused` 时与切换点交互请求结束当天。
  UI 是独立的 BSN 场景；进入时卸载当前 Road 及 Player，不提供 Esc 或 Cancel。

地图显示完整路网，法院为金色、当前路段为蓝色，可前往路段有绿色边框，
其余节点仅展示。连线直接连接节点中心，地图按窗口短边等比缩放。
`src/level/map.rs` 只读取 `RoadGraph`，每次进入 `Playing` 时根据本局路网重新生成 `MapLayout`：
法院位于原点，将路网视为无向图做 BFS，同层路段按稳定的 ID 顺序均匀分布在圆环上；
正常路网假设与法院连通。布局只负责坐标，实际通行仍由有向路网和导航校验决定。
`ui.rs` 使用原生 UI / BSN 绘制节点和连线，并复用原有目的地选择逻辑。

`InitialRoad` Resource 保存初始路段，目前固定为 `RoadId::ExampleRoad`。
新游戏从周六开始，点击开始按钮后直接显示路段解锁地图。
`RoadUnlocks(HashMap<RoadId, bool>)` 保存本局解锁状态，初始值来自 `roads.graph.ron`：
ExampleRoad 和 Court 为 `true`，MarketRoad 及 EmptyRoad1～EmptyRoad7 为 `false`。
解锁状态跨天保留，新游戏重新初始化。
工作日从初始路段及 Player 开始，只能沿有向路网选择已解锁的相邻路段：
ExampleRoad 可前往法院，MarketRoad 解锁后也可从 ExampleRoad 前往，再前往法院。
周末地图展示完整路网，只允许选择已解锁路网的相邻未解锁路段；选择后立即解锁并进入该路段，
保持 `CommuteState::Paused`，正常移动和交互，停止通勤计时。
MarketRoad 与七个空路段直接双向连接；空路段共用只有玩家、地面和切换点的 BSN 场景。
加上 MarketRoad，默认 28 天的八个周末日各有一个路段可以解锁，通过路段切换点结束当天。
开始菜单、Road 和地图 UI 的根实体统一由 `SceneRoot` 标记；每次切换删除旧场景根实体
及全部子实体，再生成新场景。Camera 始终保留；开始游戏时生成左上角日期 HUD 和右上角通勤 HUD，
之后切换 Road 与地图 UI 时保留。
每个 Road 通过 `player_scene(Vec2::new(-460.0, -130.0))` 定义自己的 Player
子场景和出生位置；进入新 Road 时生成新的 Player，移动、跳跃和物理逻辑照常运行。
Player 不存在时，移动和交互系统自然跳过。
法院通过 `court_scene()` 定义为普通路段，根实体标识为 `RoadId::Court`，
中央棕色法官席复用建筑的碰撞感应区域与 E 键交互。工作日靠近后按 E，底部显示
“请作出你的判决”，再次按 E 后显示有罪、无罪和离开按钮。
法官席左侧为红色公诉方，右侧为蓝色辩护方，靠近按 E 可查看示例陈述，
再次按 E 结束陈述。三者直接使用参数化的 `dialogue_building_scene` 创建，
通过 `BuildingDialogue` 指定 Replique 资源和节点，复用统一对话启动 Observer。
工作日进入法院保持 `Paused`，法院对话和 `verdict()` 判决照常执行。
法院初始已解锁，周末解锁地图不会提供进入法院的选项。
工作日有罪判决为 ExampleRoad 添加高空抛物，无罪判决为 MarketRoad 添加施工绕道，
同一种效果不会重复添加。判决先保存效果并请求结束当天，等待 Replique 对话结束及清理后再换天；
周五先显示账单，列出当前余额、本周工资、房租和结算后余额，按 E 确认后结算并换天。
每周只结算一次，余额可以为负；最后一天为周五时，也须确认账单后才显示实习结束。
“离开”仅关闭对话，可以再次交互，不结束当天。
对话期间停止水平移动、跳跃和冲刺消耗，保留重力与物理模拟，关闭后恢复操作。
`assets/dialogue/court.rep` 和 `assets/dialogue/buildings.rep` 通过 `bevy_replique`
管理法院、商店和猫咪的文案、分支及结束；工作日法院使用布尔参数判决命令。
`src/level/dialogue.rs` 和 `src/level/dialogue/ui.rs` 负责共用对话逻辑与 BSN 对话框。
对话框、账单和实习结束界面使用随项目提供的全量中文字体，来源与许可证见 `assets/fonts/`；其余 UI 使用默认字体。
对话推进与账单确认共用 `UiAction::Confirm`，默认绑定 E，`InputMap<UiAction>` 位于各自 BSN 根实体。
新界面会等待前一个界面的 E 松开再接受确认，避免一次按键连续推进。

静态配置位于 `assets/`，修改对应 RON 文件后重新运行即可生效：

- `game.settings.ron`：`total_days` 为实习总天数，默认 28；`commute_limit` 为通勤时限，`initial_money` 为初始金钱，
  `weekly_salary` 为每周工资（默认 100），`weekly_rent` 为每周房租（默认 80），
  `multiplier` 为冲刺速度倍率，`per_second` 为每秒体力消耗。
- `roads.graph.ron`：`roads` 按 `RoadId` 枚举值配置下一路段数组；空数组表示没有目的地。
  `initial_unlocks` 按 `RoadId` 配置新游戏的解锁状态。
- `buildings.values.ron`：`cat_reward` 为统一猫咪奖励；`lottery` 配置彩票 `price`
  及 `prizes`，每项使用 `reward` 和 `weight` 定义奖励与相对权重；
  `convenience_stores` 按 Small、Large 类型配置 `price` 和 `stamina_recovery`。
- `cases.ron`：保存 `CaseDef` 列表，每项包含 `dialogue_prefix`、`guilty`、`not_guilty`。
  公诉和辩护台词对应 `dialogue/court.rep` 中的 `<dialogue_prefix>_prosecution`、
  `<dialogue_prefix>_defense` 节点，共用 `verdict` 节点作出判决。

`src/config.rs` 集中定义配置类型与 `ConfigPlugin` 加载逻辑，使用
`bevy_asset_loader` 加载、`bevy_common_assets` 反序列化 RON。启动时进入
`GameState::Loading`，全部配置加载完成并从静态 Asset 转为只读 Resource 后，
进入 `GameState::MainMenu`，通过 Bevy UI / BSN 显示开始菜单。点击开始按钮后进入
`GameState::Playing`，初始化本局数据、HUD 和周末解锁地图。
开始菜单随状态切换清理。系统直接读取
`GameSettings`、`RoadGraph`、`BuildingValues`、`CaseDefs`，无需反复查询 `Assets<T>`。

`src/game.rs` 的 `GamePlugin` 管理全局 `CommuteTime`、`Money`、`Stamina`、
`HasHelmet`、`GameCalendar` 和 `CommuteState`。固定参数直接读取配置，`Money` 和
`CommuteTime.elapsed` 仍是运行时资源；每次进入 `Playing`，金钱按 `initial_money`
初始化，已用时间重置为 0、体力恢复至上限、初始没有安全帽。`Stamina::MAX` 保持硬编码为 100。
体力、安全帽和金钱保存在全局资源中，切换路段或重新生成 Player 不会重置。
工作日进入初始 Road 后开始通勤计时；目的地选择 UI 请求 `Paused`，
选择普通 Road 后恢复 `Running`，进入 Court 后保持 `Paused`。周末地图和选定路段都保持 `Paused`，不计时。
状态请求由 Bevy 在下一次状态切换调度中应用，不影响物理和玩家移动。
超时后已用时间继续增长，`remaining(limit)` 接收配置中的通勤时限，返回时限减去
已用时间，可为负数。
HUD 独立于 `SceneRoot`，实时显示剩余秒数、金钱和可视化体力条，
在全屏目的地 UI 上也保持可见。

`src/game/calendar.rs` 集中日期资源和日期 HUD。`GameCalendar` 保存 `total_days` 和 `elapsed_days`，新局从已过 0 天、周六开始；
`weekday()` 根据已过天数返回周一至周日的 1～7，`remaining_days()` 返回剩余天数，
`is_weekend()` 判断周末。独立的左上角日期 HUD 随日期变化更新，切换关卡或换天时保留。
`src/level/day.rs` 的 `finish_day` 在 `Last` 消费换天请求，等待 `PostUpdate` 完成对话清理，
周五进入 `src/level/bill.rs` 的 BSN 账单场景，卸载道路和 Player 并暂停计时。
按 E 后执行 `Money += weekly_salary - weekly_rent`，复用统一的 `begin_next_day`；其余日期直接换天。
`begin_next_day` 在换天时
将已过天数加 1、通勤时间清零、体力恢复上限，
替换当前 `SceneRoot`，保留金钱、安全帽、道路效果和解锁状态。
`start_day` 通过现有 `GameCalendar::is_weekend()` 决定当天入口：
周末显示解锁地图，工作日生成 `InitialRoad` 及全新 Player 并恢复通勤计时，法院仍保持 `Paused`。
换天保持 `GameState::Playing`，不新增周末游戏状态。
最后一天结束后改为显示“实习结束”并暂停计时，不再生成关卡或玩家，暂不判定正式胜败。
结束场景及其标记组件位于 `src/level/scene/ending.rs`；导航和换天共用 `scene.rs` 的 `replace_scene`。

关卡场景按平台、共用建筑、各类建筑、切换点、效果和路段组合拆分到
`src/level/scene/`，由 `scene.rs` 组织模块。场景使用 Bevy 0.20 的 `bsn!`、
`@scene(...)` 和 `Children [ ... -- ... ]` 组合。

路段效果的数据、布局和行为分别维护：

- `RoadEffects(HashMap<RoadId, Vec<RoadEffect>>)` 保存本局永久生效的效果。
  每次开始新局时初始化为空，之后通过法院判决添加；切换路段或打开地图不会改变这些数据。
- `src/level/scene/road/` 中每个路段独立定义布局，接收该路段的效果数组，
  自行指定坠落物候选横坐标、路障位置及效果场景组合。
- `src/level/scene/effect/` 复用 BSN 效果场景及坠落碰撞行为。方块外观和 Collider
  使用相同尺寸；坠落物使用动态刚体，施工路障使用静态刚体，直接由物理碰撞阻挡通行。
  效果实体均为路段根实体的子实体，随路段一起销毁，再次进入时根据永久效果重新生成。

玩家的 `Stunned` 组件只记录剩余眩晕时间，原移动系统通过 `Without<Stunned>`
跳过眩晕玩家，倒计时结束移除组件后自动恢复操作。

`TodayRoads(HashSet<RoadId>)` 每天重置，记录实际进入的路段，包含当天生成的初始路段，
排除法院和地图上未选择的目的地。进入法院时，先从当天经过且仍有可用案件的路段中均匀抽取一路，
再均匀抽取该路段的可用案件；案件的两种效果均未在该路段出现时才可用。
`CurrentCase(Option<SelectedCase>)` 保存目标 `RoadId` 和完整 `CaseDef`，查看双方台词或选择离开时保留。
`verdict(bool)` 将对应效果添加到目标路段（不重复添加），立即清空当前案件并请求结束当天。
当前配置提供四个案件；`PlaceholderA`、`PlaceholderB` 仅保存为路段效果，不生成场景行为。
抽取前提是当天存在可用案件，由玩家路线保证，不处理案件不足。

`platform_scene(position, size)` 的尺寸写入 `Platform` 组件，并同时用于 Sprite
和 Collider，方便以后替换为九切片或平铺贴图。便利店、工具店、彩票店和猫咪
各自有场景函数和 `BuildingType` 组件，共用参数化的 `dialogue_building_scene`：
接受位置、大小、颜色和 `BuildingDialogue`，统一组合感应区、外观和对话 Observer。
商店与猫咪在 `Paused` 时启动信息对话，`Running` 时保留原有交易或救援行为。
便利店实体通过 `ConvenienceKind` 指定 Small 或 Large，其余同类建筑共享配置。
工具店保持固定花费 20 元购买安全帽，不加入配置。`RoadId` 是支持反序列化的枚举，
`SwitchPoint` 是标记组件，由 `navigation.rs` 根据所在路段的 `RoadId` 查询
`RoadGraph` 中的可选目的地。

新增路段时，在 `src/level/scene/road/` 下按 `example_road_scene` 的形式定义
接收效果数组、返回 `impl Scene` 的函数，
为 `RoadId` 增加枚举变体并分配给根实体，在 `src/level/scene.rs` 的 `road_scene`
入口中登记 ID 与场景函数的对应关系，再编辑 `assets/roads.graph.ron`，添加新路段、
初始解锁状态及前一路段可前往的目的地。`navigation.rs` 负责目的地验证、场景切换及通勤状态请求；无效目的地、重复或
已失效的交互不会卸载当前场景或重复生成场景。
子实体的位置相对于根实体；平台用尺寸指定形状，根实体保持单位缩放。

建筑整体和切换点使用与占位外观同尺寸的 `Collider + Sensor + CollidingEntities`
检测重叠，允许玩家穿过。统一交互逻辑位于 `src/level/interaction.rs`，
每次新按下 E，根据 `CommuteState` 向与玩家重叠的可交互目标发送一次
`RunningInteractRequested` 或 `PausedInteractRequested` EntityEvent，
事件包含目标和玩家实体，不处理多个交互区域重叠的情况。建筑通过 BSN 的 `on(...)` 绑定统一对话
Observer 和各自的工作日效果：所有建筑（包括法院角色）只响应暂停交互启动对话，普通建筑只响应运行交互执行交易或救援。
切换点分别监听两种事件以导航或结束当天；路段切换由 `navigation.rs` 处理，目的地选择界面位于 `ui.rs`。
暂不包含随机路网生成、每日猫咪刷新、跨场景救援状态保存、
公交站或实际判案逻辑。

验证：`cargo test`、`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`。
仅运行效果生命周期测试可用 `cargo test road_effects`，验证新局初始化和重置、
切换场景的效果清理、永久效果保留，以及再次进入路段时重新生成效果。

测试以游戏行为为边界：配置加载测试读取真实 RON 文件，其余测试使用独立数据，
避免调整数值时连带修改测试。移动、交易和导航规则使用轻量测试；真实键盘、
碰撞和完整通勤流程保留无窗口集成测试。地图与 HUD 验证显示内容和可选目的地，
不固定颜色、像素尺寸或场景内部组件清单。
