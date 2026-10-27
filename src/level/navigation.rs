use super::{
    RoadId, SceneRoot, SwitchPoint, scene,
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
        if !destinations.contains(&destination) {
            destinations.push(destination);
        }
    }
    if !destinations.is_empty() {
        replace_scene(world, root, ui::travel_ui_scene(destinations));
        world
            .resource_mut::<NextState<CommuteState>>()
            .set(CommuteState::Paused);
    }
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
    if choices.0.contains(&destination) {
        replace_scene(world, root, scene::road_scene(destination));
        let state = if destination == scene::COURT {
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

fn replace_scene(world: &mut World, root: Entity, next: impl Scene) {
    // Exclusive commands validate their source at execution time, so stale events do nothing.
    world.despawn(root);
    world.spawn_scene(next).unwrap();
}
