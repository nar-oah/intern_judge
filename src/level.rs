mod interaction;
mod map;
mod menu;
mod navigation;
mod scene;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod travel_tests;
mod ui;
use crate::config::{GameState, RoadGraph};
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Component, Clone, Default)]
struct SceneRoot;

#[derive(
    Component, Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub enum RoadId {
    #[default]
    ExampleRoad,
    MarketRoad,
    Court,
}

#[derive(Resource, Default)]
pub struct InitialRoad(pub RoadId);

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Platform {
    pub size: Vec2,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BuildingType {
    #[default]
    ConvenienceStore,
    ToolStore,
    LotteryStore,
    Cat,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SwitchPoint;

#[derive(Component, Clone, Copy, Default)]
struct Interactable;

#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct InteractRequested {
    pub entity: Entity,
    pub player: Entity,
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InitialRoad>()
            .add_systems(OnEnter(GameState::MainMenu), menu::setup_menu)
            .add_systems(OnExit(GameState::MainMenu), menu::cleanup_menu)
            .add_systems(OnEnter(GameState::Playing), |world: &mut World| {
                let layout = map::MapLayout::from_graph(world.resource::<RoadGraph>());
                world.insert_resource(layout);
                let road = world.resource::<InitialRoad>().0;
                world.spawn_scene(scene::road_scene(road)).unwrap();
            })
            .add_systems(
                PostUpdate,
                interaction::interact.run_if(in_state(GameState::Playing)),
            );
    }
}
