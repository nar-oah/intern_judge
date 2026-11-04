#[cfg(test)]
mod day_tests;
mod day;
mod dialogue;
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
use crate::{
    config::{GameState, RoadGraph},
};
use bevy::prelude::*;
use bevy_replique::prelude::{DialogueRunner, DialogueSystem};
use serde::Deserialize;
use std::collections::HashMap;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoadEffect {
    FallingObject,
    ConstructionDetour,
}

#[derive(Resource, Debug, Default, PartialEq, Eq)]
pub struct RoadEffects(pub HashMap<RoadId, Vec<RoadEffect>>);

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
        dialogue::setup(app);
        app.init_resource::<InitialRoad>()
            .add_systems(OnEnter(GameState::MainMenu), menu::setup_menu)
            .add_systems(OnExit(GameState::MainMenu), menu::cleanup_menu)
            .add_systems(OnEnter(GameState::Playing), |world: &mut World| {
                world.remove_resource::<day::DayEndRequested>();
                world.insert_resource(RoadEffects::default());
                let layout = map::MapLayout::from_graph(world.resource::<RoadGraph>());
                world.insert_resource(layout);
                let road = world.resource::<InitialRoad>().0;
                let effects = world.resource::<RoadEffects>().0.get(&road);
                let scene = scene::road_scene(road, effects.map_or(&[], Vec::as_slice));
                world.spawn_scene(scene).unwrap();
            })
            .add_systems(
                Last,
                day::begin_next_day
                    .run_if(resource_exists::<day::DayEndRequested>)
                    .run_if(not(any_with_component::<DialogueRunner>))
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                interaction::interact
                    .before(DialogueSystem::Runner)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

