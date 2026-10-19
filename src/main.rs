mod config;
mod level;
mod player;
mod player_scene;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use config::{GameAssets, GameState, InputConfig, PlayerConfig};
use level::LevelPlugin;
use player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default().with_length_unit(20.0),
            // Different RON asset types must have distinct file suffixes.
            RonAssetPlugin::<PlayerConfig>::new(&["player.ron"]),
            RonAssetPlugin::<InputConfig>::new(&["input.ron"]),
            PlayerPlugin,
            LevelPlugin,
        ))
        .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Playing)
                .load_collection::<GameAssets>(),
        )
        .insert_resource(Gravity(Vec2::NEG_Y * 1000.0))
        .insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.13)))
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
