use super::{
    RoadId, SceneRoot, SwitchPoint, scene,
    ui::{self, TravelButtonAction, TravelUiRoot},
};
use crate::player::Player;
use bevy::prelude::*;

pub(super) fn open_destinations(world: &mut World, point: Entity, player: Entity) {
    let Some(root) = scene_root(world, point) else {
        return;
    };
    if world.get::<RoadId>(root).is_none()
        || world.get::<Player>(player).is_none()
        || scene_root(world, player) != Some(root)
    {
        return;
    }
    let Some(point) = world.get::<SwitchPoint>(point) else {
        return;
    };
    let mut destinations = Vec::new();
    for &destination in point.destinations {
        if !destinations.contains(&destination) && scene::road_scene(destination).is_some() {
            destinations.push(destination);
        }
    }
    if !destinations.is_empty() {
        replace_scene(world, root, ui::travel_ui_scene(destinations));
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
    if choices.0.contains(&destination)
        && let Some(road) = scene::road_scene(destination)
    {
        replace_scene(world, root, road);
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
