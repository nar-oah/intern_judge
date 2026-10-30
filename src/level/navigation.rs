use super::{
    RoadEffects, RoadId, SceneRoot, SwitchPoint,
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
        if !destinations.contains(&destination) {
            destinations.push(destination);
        }
    }
    if !destinations.is_empty() {
        let next = ui::travel_ui_scene(
            road,
            destinations,
            world.resource::<RoadGraph>(),
            world.resource::<MapLayout>(),
        );
        replace_scene(world, root, next);
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
        let effects = world.resource::<RoadEffects>().0.get(&destination);
        let next = scene::road_scene(destination, effects.map_or(&[], Vec::as_slice));
        replace_scene(world, root, next);
        let state = if destination == RoadId::Court {
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
    world.despawn(root);
    world.spawn_scene(next).unwrap();
}
