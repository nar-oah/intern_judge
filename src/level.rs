#[cfg(test)]
mod day_tests;
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
    game::{CommuteState, CommuteTime, GameCalendar, Stamina},
};
use bevy::prelude::*;
use bevy::text::FontSourceTemplate;
use bevy_replique::prelude::{DialogueRunner, DialogueSystem};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Component, Clone, Default)]
struct SceneRoot;

/// Kept until dialogue cleanup has completed, then consumed by `begin_next_day`.
#[derive(Resource)]
pub(crate) struct DayEndRequested;

#[derive(Component, Clone, Default)]
pub(crate) struct InternshipEnded;

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
                world.remove_resource::<DayEndRequested>();
                world.insert_resource(RoadEffects::default());
                let layout = map::MapLayout::from_graph(world.resource::<RoadGraph>());
                world.insert_resource(layout);
                let road = world.resource::<InitialRoad>().0;
                let effects = world.resource::<RoadEffects>().0.get(&road);
                let scene = scene::road_scene(road, effects.map_or(&[], Vec::as_slice));
                world.spawn_scene(scene).unwrap();
            })
            .add_systems(
                PostUpdate,
                begin_next_day
                    .after(dialogue::close_dialogue)
                    .run_if(resource_exists::<DayEndRequested>)
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

/// Finish the current day without entering `Playing` again or resetting session data.
pub(crate) fn begin_next_day(world: &mut World) {
    world.remove_resource::<DayEndRequested>();
    let mut calendar = world.resource_mut::<GameCalendar>();
    if calendar.is_finished() {
        return;
    }
    calendar.elapsed_days += 1;
    let finished = calendar.is_finished();
    world.insert_resource(CommuteTime::default());
    world.insert_resource(Stamina::default());

    let next = if finished {
        Box::new(internship_ended_scene()) as Box<dyn Scene>
    } else {
        let road = world.resource::<InitialRoad>().0;
        let effects = world.resource::<RoadEffects>().0.get(&road);
        scene::road_scene(road, effects.map_or(&[], Vec::as_slice))
    };
    let roots = world
        .query_filtered::<Entity, With<SceneRoot>>()
        .iter(world)
        .collect::<Vec<_>>();
    // SceneRoot has the same lifetime for roads, destination maps and the ending UI.
    if let Some((&root, others)) = roots.split_first() {
        for &other in others {
            world.despawn(other);
        }
        navigation::replace_scene(world, root, next);
    } else {
        world.spawn_scene(next).unwrap();
    }
    world
        .resource_mut::<NextState<CommuteState>>()
        .set(if finished {
            CommuteState::Paused
        } else {
            CommuteState::Running
        });
}

fn internship_ended_scene() -> impl Scene {
    bsn! {
        #InternshipEnded
        SceneRoot
        InternshipEnded
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        BackgroundColor(Color::srgb(0.04, 0.05, 0.08))
        Children [
            Text::new("实习结束")
            TextFont {
                font: FontSourceTemplate::Handle("fonts/court-dialogue.ttf"),
                font_size: FontSize::Px(48.0),
            }
            TextColor(Color::srgb(0.95, 0.96, 1.0))
            Pickable::IGNORE
        ]
    }
}
