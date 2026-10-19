use avian2d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::config::{GameAssets, GameState, InputConfig, PlayerConfig};
use crate::player_scene::player_scene;

/// Live ECS data used by player_movement; initialized from player.ron.
#[derive(Component, Clone, Default)]
pub(crate) struct Player {
    pub move_speed: f32,
    pub jump_speed: f32,
}

/// Gameplay actions are Rust types; their default key bindings live in input.ron.
#[derive(Actionlike, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub(crate) enum PlayerAction {
    Left,
    Right,
    Jump,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                FixedUpdate,
                player_movement.run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_player(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    player_configs: Res<Assets<PlayerConfig>>,
    input_configs: Res<Assets<InputConfig>>,
) {
    // LoadingState has already finished loading these two assets.
    let config = player_configs
        .get(&game_assets.player)
        .expect("player.ron should be loaded");
    let bindings = input_configs
        .get(&game_assets.input)
        .expect("input.ron should be loaded");

    let mut input_map = InputMap::<PlayerAction>::default();
    for &key in &bindings.left {
        input_map.insert(PlayerAction::Left, key);
    }
    for &key in &bindings.right {
        input_map.insert(PlayerAction::Right, key);
    }
    for &key in &bindings.jump {
        input_map.insert(PlayerAction::Jump, key);
    }

    // The player entity and its physics components are assembled in BSN.
    commands.spawn_scene(player_scene(config, input_map));
}

fn player_movement(
    mut players: Query<(
        &Player,
        &ActionState<PlayerAction>,
        &ShapeHits,
        &mut LinearVelocity,
    )>,
) {
    for (player, actions, hits, mut velocity) in &mut players {
        let left = actions.pressed(&PlayerAction::Left);
        let right = actions.pressed(&PlayerAction::Right);
        let direction = i8::from(right) - i8::from(left);
        velocity.x = f32::from(direction) * player.move_speed;

        let grounded = hits.iter().any(|hit| (-hit.normal2).dot(Vec2::Y) > 0.7);
        if grounded && actions.just_pressed(&PlayerAction::Jump) {
            velocity.y = player.jump_speed;
        }
    }
}
