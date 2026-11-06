mod bill;
mod case;
mod day;
mod dialogue;
mod interaction;
mod map;
mod menu;
mod navigation;
mod scene;
#[cfg(test)]
#[path = "../tests/level/mod.rs"]
mod tests;
mod ui;
use crate::config::{CaseDef, GameState, RoadGraph};
use bevy::prelude::*;
use bevy_replique::prelude::{DialogueRunner, DialogueSystem};
use leafwing_input_manager::prelude::InputManagerPlugin;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

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
    EmptyRoad1,
    EmptyRoad2,
    EmptyRoad3,
    EmptyRoad4,
    EmptyRoad5,
    EmptyRoad6,
    EmptyRoad7,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum RoadEffect {
    FallingObject,
    ConstructionDetour,
    PlaceholderA,
    PlaceholderB,
}

#[derive(Resource, Debug, Default, PartialEq, Eq)]
pub struct RoadEffects(pub HashMap<RoadId, Vec<RoadEffect>>);

#[derive(Resource, Debug, Default, PartialEq, Eq)]
pub struct TodayRoads(pub HashSet<RoadId>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedCase {
    pub road: RoadId,
    pub case: CaseDef,
}

#[derive(Resource, Debug, Default, PartialEq, Eq)]
pub struct CurrentCase(pub Option<SelectedCase>);

#[derive(Resource, Debug, Default, PartialEq, Eq)]
pub struct RoadUnlocks(pub HashMap<RoadId, bool>);

#[derive(Resource, Default)]
pub struct InitialRoad(pub RoadId);

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Platform {
    pub size: Vec2,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SwitchPoint;

#[derive(Component, Clone, Copy, Default)]
struct Interactable;

#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct RunningInteractRequested {
    pub entity: Entity,
    pub player: Entity,
}

#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct PausedInteractRequested {
    pub entity: Entity,
    pub player: Entity,
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        dialogue::setup(app);
        app.add_plugins(InputManagerPlugin::<ui::UiAction>::default())
            .init_resource::<InitialRoad>()
            .add_systems(OnEnter(GameState::MainMenu), menu::setup_menu)
            .add_systems(OnExit(GameState::MainMenu), menu::cleanup_menu)
            .add_systems(
                OnEnter(GameState::Playing),
                (|world: &mut World| {
                    world.remove_resource::<day::DayEndRequested>();
                    world.insert_resource(RoadEffects::default());
                    let unlocks = world.resource::<RoadGraph>().initial_unlocks.clone();
                    world.insert_resource(RoadUnlocks(unlocks));
                    let layout = map::MapLayout::from_graph(world.resource::<RoadGraph>());
                    world.insert_resource(layout);
                    day::start_day(world);
                })
                .after(crate::game::calendar::initialize_calendar),
            )
            .add_systems(
                Last,
                day::finish_day
                    .run_if(resource_exists::<day::DayEndRequested>)
                    .run_if(not(any_with_component::<DialogueRunner>))
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                bill::confirm_bill
                    .run_if(any_with_component::<bill::WeeklyBill>)
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
