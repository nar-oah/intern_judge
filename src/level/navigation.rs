use super::{
    RoadId, RoadUnlocks, RunningInteractRequested, SceneRoot,
    map::MapLayout,
    scene,
    ui::{self, TravelButtonAction, TravelUiRoot},
};
use crate::{config::RoadGraph, game::CommuteState};
use bevy::ui_widgets::Activate;
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(SystemParam)]
pub(super) struct NavigationMap<'w> {
    graph: Res<'w, RoadGraph>,
    unlocks: Res<'w, RoadUnlocks>,
    layout: Res<'w, MapLayout>,
}

pub(super) fn open_destinations(
    request: On<RunningInteractRequested>,
    mut commands: Commands,
    roots: Query<&RoadId, With<SceneRoot>>,
    parents: Query<&ChildOf>,
    map: NavigationMap,
    mut commute: ResMut<NextState<CommuteState>>,
) {
    let Some(root) = parents
        .iter_ancestors(request.entity)
        .find(|&entity| roots.contains(entity))
    else {
        return;
    };
    let Ok(&road) = roots.get(root) else {
        return;
    };
    let Some(choices) = map.graph.roads.get(&road) else {
        return;
    };
    let mut destinations = Vec::new();
    for &destination in choices {
        if map.unlocks.0.get(&destination) == Some(&true) && !destinations.contains(&destination) {
            destinations.push(destination);
        }
    }
    if !destinations.is_empty() {
        commands.entity(root).despawn();
        commands.spawn_scene(ui::travel_ui_scene(
            Some(road),
            destinations,
            &map.graph,
            &map.layout,
        ));
        commute.set(CommuteState::Paused);
    }
}

#[derive(Event)]
pub(super) struct OpenUnlockMap;

pub(super) fn open_unlock_map(
    _: On<OpenUnlockMap>,
    mut commands: Commands,
    roots: Query<Entity, With<SceneRoot>>,
    map: NavigationMap,
    mut commute: ResMut<NextState<CommuteState>>,
) {
    let unlocks = &map.unlocks.0;
    let mut destinations = map
        .graph
        .roads
        .iter()
        .filter(|(road, _)| unlocks.get(*road) == Some(&true))
        .flat_map(|(_, neighbors)| neighbors.iter().copied())
        .filter(|road| unlocks.get(road) == Some(&false))
        .collect::<Vec<_>>();
    destinations.sort_unstable();
    destinations.dedup();
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.spawn_scene(ui::travel_ui_scene(
        None,
        destinations,
        &map.graph,
        &map.layout,
    ));
    commute.set(CommuteState::Paused);
}

pub(super) fn select_destination(
    activate: On<Activate>,
    mut commands: Commands,
    buttons: Query<&TravelButtonAction>,
    parents: Query<&ChildOf>,
    maps: Query<&TravelUiRoot, With<SceneRoot>>,
    mut unlocks: ResMut<RoadUnlocks>,
    mut commute: ResMut<NextState<CommuteState>>,
) {
    let Ok(&TravelButtonAction(destination)) = buttons.get(activate.entity) else {
        return;
    };
    let Some(choices) = parents
        .iter_ancestors(activate.entity)
        .find_map(|entity| maps.get(entity).ok())
    else {
        return;
    };
    let unlocking = choices.1;
    if choices.0.contains(&destination) && unlocks.0.get(&destination) == Some(&!unlocking) {
        if unlocking {
            unlocks.0.insert(destination, true);
        }
        commands.trigger(scene::EnterRoad(destination));
        let state = if unlocking || destination == RoadId::Court {
            CommuteState::Paused
        } else {
            CommuteState::Running
        };
        commute.set(state);
    }
}
