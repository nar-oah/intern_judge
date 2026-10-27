#[cfg(test)]
mod tests;

use crate::level::RoadId;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
}

#[derive(Asset, Resource, TypePath, Clone, Debug, Deserialize)]
pub struct GameSettings {
    pub commute_limit: f32,
    pub initial_money: i32,
    pub sprint_multiplier: f32,
    pub stamina_drain_per_second: f32,
}

#[derive(Asset, Resource, TypePath, Clone, Debug, Deserialize)]
pub struct RoadGraph {
    pub roads: HashMap<RoadId, Vec<RoadId>>,
}

#[derive(Component, Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
pub enum ConvenienceKind {
    Small,
    Large,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConvenienceValues {
    pub price: i32,
    pub stamina_recovery: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LotteryPrize {
    pub reward: i32,
    pub weight: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LotteryValues {
    pub price: i32,
    pub prizes: Vec<LotteryPrize>,
}

#[derive(Asset, Resource, TypePath, Clone, Debug, Deserialize)]
pub struct BuildingValues {
    pub cat_reward: i32,
    pub lottery: LotteryValues,
    pub convenience_stores: HashMap<ConvenienceKind, ConvenienceValues>,
}

#[derive(AssetCollection, Resource)]
struct ConfigAssets {
    #[asset(path = "game.settings.ron")]
    settings: Handle<GameSettings>,
    #[asset(path = "roads.graph.ron")]
    roads: Handle<RoadGraph>,
    #[asset(path = "buildings.values.ron")]
    buildings: Handle<BuildingValues>,
}

impl FromWorld for GameSettings {
    fn from_world(world: &mut World) -> Self {
        let handle = &world.resource::<ConfigAssets>().settings;
        world
            .resource::<Assets<Self>>()
            .get(handle)
            .unwrap()
            .clone()
    }
}

impl FromWorld for RoadGraph {
    fn from_world(world: &mut World) -> Self {
        let handle = &world.resource::<ConfigAssets>().roads;
        world
            .resource::<Assets<Self>>()
            .get(handle)
            .unwrap()
            .clone()
    }
}

impl FromWorld for BuildingValues {
    fn from_world(world: &mut World) -> Self {
        let handle = &world.resource::<ConfigAssets>().buildings;
        world
            .resource::<Assets<Self>>()
            .get(handle)
            .unwrap()
            .clone()
    }
}

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                RonAssetPlugin::<GameSettings>::new(&["settings.ron"]),
                RonAssetPlugin::<RoadGraph>::new(&["graph.ron"]),
                RonAssetPlugin::<BuildingValues>::new(&["values.ron"]),
            ))
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .load_collection::<ConfigAssets>()
                    .finally_init_resource::<GameSettings>()
                    .finally_init_resource::<RoadGraph>()
                    .finally_init_resource::<BuildingValues>()
                    .continue_to_state(GameState::Playing),
            );
    }
}
