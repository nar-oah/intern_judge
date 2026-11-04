use super::{
    RoadEffects, RoadId, RoadUnlocks, SceneRoot, SwitchPoint,
    map::MapLayout,
    scene,
    ui::{self, TravelButtonAction, TravelUiRoot},
};
use crate::{config::RoadGraph, game::CommuteState, player::Player};
use bevy::prelude::*;

pub(super) fn open_destinations(world: &mut World, point: Entity, player: Entity) {
    let Some(root) = scene_root(world, point) else {
        return;
    };
    let Some(&road) = world.get::<RoadId>(root) else {
        return;
    };
    if world.get::<Player>(player).is_none()
        || scene_root(world, player) != Some(root)
        || world.get::<SwitchPoint>(point).is_none()
    {
        return;
    }
    let Some(choices) = world.resource::<RoadGraph>().roads.get(&road) else {
        return;
    };
    let mut destinations = Vec::new();
    for &destination in choices {
        if world.resource::<RoadUnlocks>().0.get(&destination) == Some(&true)
            && !destinations.contains(&destination)
        {
            destinations.push(destination);
        }
    }
    if !destinations.is_empty() {
        let next = ui::travel_ui_scene(
            Some(road),
            destinations,
            world.resource::<RoadGraph>(),
            world.resource::<MapLayout>(),
        );
        scene::replace_scene(world, [root], next);
        world
            .resource_mut::<NextState<CommuteState>>()
            .set(CommuteState::Paused);
    }
}

pub(super) fn open_unlock_map(world: &mut World) {
    let graph = world.resource::<RoadGraph>();
    let unlocks = &world.resource::<RoadUnlocks>().0;
    let mut destinations = graph
        .roads
        .iter()
        .filter(|(road, _)| unlocks.get(road) == Some(&true))
        .flat_map(|(_, neighbors)| neighbors.iter().copied())
        .filter(|road| unlocks.get(road) == Some(&false))
        .collect::<Vec<_>>();
    destinations.sort_unstable();
    destinations.dedup();
    let next = ui::travel_ui_scene(None, destinations, graph, world.resource::<MapLayout>());
    let roots = world
        .query_filtered::<Entity, With<SceneRoot>>()
        .iter(world)
        .collect::<Vec<_>>();
    scene::replace_scene(world, roots, next);
    world
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Paused);
}

pub(super) fn select_destination(world: &mut World, button: Entity) {
    let Some(&TravelButtonAction(destination)) = world.get::<TravelButtonAction>(button) else {
        return;
    };
    let Some(root) = scene_root(world, button) else {
        return;
    };
    let Some(choices) = world.get::<TravelUiRoot>(root) else {
        return;
    };
    let unlocking = choices.1;
    if choices.0.contains(&destination)
        && world.resource::<RoadUnlocks>().0.get(&destination) == Some(&!unlocking)
    {
        if unlocking {
            world
                .resource_mut::<RoadUnlocks>()
                .0
                .insert(destination, true);
        }
        let effects = world.resource::<RoadEffects>().0.get(&destination);
        let next = scene::road_scene(destination, effects.map_or(&[], Vec::as_slice));
        scene::replace_scene(world, [root], next);
        let state = if unlocking || destination == RoadId::Court {
            CommuteState::Paused
        } else {
            CommuteState::Running
        };
        world.resource_mut::<NextState<CommuteState>>().set(state);
    }
}

fn scene_root(world: &World, mut entity: Entity) -> Option<Entity> {
    loop {
        if world.get::<SceneRoot>(entity).is_some() {
            return Some(entity);
        }
        entity = world.get::<ChildOf>(entity)?.parent();
    }
}
