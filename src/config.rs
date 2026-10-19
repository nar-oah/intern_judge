use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use serde::Deserialize;

/// Assets load first; the player is spawned after entering Playing.
#[derive(States, Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
}

/// Tuning data (like a Godot .tres), not a live player ECS component.
#[derive(Asset, TypePath, Deserialize)]
pub struct PlayerConfig {
    pub move_speed: f32,
    pub jump_speed: f32,
    pub size: [f32; 2],
}

/// Default key bindings; runtime button state belongs to Leafwing ActionState.
#[derive(Asset, TypePath, Deserialize)]
pub struct InputConfig {
    pub left: Vec<KeyCode>,
    pub right: Vec<KeyCode>,
    pub jump: Vec<KeyCode>,
}

/// Keeps strong handles and waits for both RON assets during Loading.
#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "config/player.ron")]
    pub player: Handle<PlayerConfig>,

    #[asset(path = "config/input.ron")]
    pub input: Handle<InputConfig>,
}
