# Intern Judge

Bevy 0.20 + Avian 0.8 的简单平台关卡示例。运行：`cargo run`。

- A / D：移动；空格：跳跃；E：与附近最近的建筑或切换点交互。
- 住宅为绿色、商店为橙色、工厂为紫色，切换点为青色；交互结果输出到日志。

关卡定义在 `src/level/scene.rs`，使用 Bevy 0.20 的 `bsn!`、`@scene(...)` 和
`Children [ ... -- ... ]` 组合。示例路段包含五个平台、三类建筑、两个切换点。

`platform_scene(position, size)` 的尺寸写入 `Platform` 组件，并同时用于 Sprite
和 Collider，方便以后替换为九切片或平铺贴图。三种建筑各自有场景函数和
`BuildingType` 组件，共用建筑占位场景；切换点通过字符串 ID 区分。

新增路段时，按 `example_road_scene` 的形式定义返回 `impl Scene` 的函数，
为根实体分配独立、固定的 `RoadId` 字符串。子实体的位置相对于根实体，
删除根实体即可一起删除平台和交互目标。平台用尺寸指定形状，根实体保持单位缩放。

统一交互逻辑位于 `src/level/interaction.rs`，在世界变换更新后检测距离；
每次新按下 E 只触发一个目标。目前只记录日志，建筑没有经济效果，切换点不切换场景。
