# 实习法官（Bevy 学习原型）

一个使用 Bevy 0.19、Avian 0.7 和 Leafwing Input Manager 0.21 的 2D 平台跳跃原型。

## 文件如何分工

| 文件 | 负责什么 | 类比 Godot |
| --- | --- | --- |
| `assets/config/player.config.ron` | 移动速度、跳跃速度、碰撞尺寸 | 可编辑的 `.tres` |
| `assets/config/input.bindings.ron` | 默认键位，可为动作配置多个按键 | Input Map 配置 |
| `src/config.rs` | RON 资产数据类型、加载集合和 Loading / Playing 状态 | 资源类型与预加载 |
| `src/player_scene.rs` | 用 `bsn!` 定义玩家的 Sprite、Transform、Avian 物理组件、InputMap | `.tscn` 的实体组合 |
| `src/player.rs` | 动作类型、从资产构建 InputMap、移动与跳跃 System | 玩家脚本 |
| `src/level.rs` | 两个临时测试平台（暂不改为 BSN） | 测试关卡 |

RON 语法提示：`PlayerConfig.size` 是 Rust 固定长度数组 `[f32; 2]`，在 RON 中须写成 `size: (30.0, 50.0)`。`InputConfig` 的键位列表是 `Vec<KeyCode>`，使用 `[...]`。

## 运行

```sh
cargo run
```

A / ← 向左，D / → 向右，Space 跳跃。

注意：两种配置使用不同的复合扩展名（`.config.ron` / `.bindings.ron`），以便 Bevy 为它们选择正确的资产加载器。

运行时，`bevy_common_assets` 将两个 RON 文件反序列化为 `Asset`；`bevy_asset_loader` 等待它们加载完毕，进入 `Playing` 后才调用 `spawn_player`。玩家的 BSN 场景由这些配置生成，`ActionState` 和 `LinearVelocity` 等运行时状态仍由 ECS / 插件管理。

**注意**：当前 RON 是初始配置。改变文件后需要重新启动游戏才能让已经生成的玩家采用新参数；尚未实现运行时热更新或用户自定义键位保存。BSN 仍写在 Rust 代码里，不是独立的 `.bsn` 资产。案件、租金和经济数据等系统尚未实现，等做对应玩法时再按同样方式添加 RON 资产。

检查代码：

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```
