use super::{GamePhase, ROAD_SPAWN, RoadId, scene};
use crate::player::Player;
use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(super) struct DestinationChoices(pub(super) &'static [RoadId]);

pub(super) fn open_destinations(world: &mut World, destinations: &'static [RoadId]) {
    if *world.resource::<State<GamePhase>>().get() != GamePhase::Exploring
        || !matches!(world.resource::<NextState<GamePhase>>(), NextState::Unchanged)
        || destinations.is_empty()
    {
        return;
    }
    world.resource_mut::<DestinationChoices>().0 = destinations;
    world
        .resource_mut::<NextState<GamePhase>>()
        .set(GamePhase::ChoosingDestination);
}

pub(super) fn stop_horizontal_motion(mut players: Query<&mut LinearVelocity, With<Player>>) {
    for mut velocity in &mut players {
        velocity.x = 0.0;
    }
}

pub(super) fn clear_destinations(mut choices: ResMut<DestinationChoices>) {
    choices.0 = &[];
}

pub(super) fn close_ui(world: &mut World) {
    if *world.resource::<State<GamePhase>>().get() == GamePhase::ChoosingDestination {
        world
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::Exploring);
    }
}

pub(super) fn select_destination(world: &mut World, destination: RoadId) {
    if *world.resource::<State<GamePhase>>().get() != GamePhase::ChoosingDestination
        || !matches!(world.resource::<NextState<GamePhase>>(), NextState::Unchanged)
        || !world.resource::<DestinationChoices>().0.contains(&destination)
    {
        return;
    }

    let roads: Vec<_> = world
        .query_filtered::<Entity, With<RoadId>>()
        .iter(world)
        .collect();
    // Resolve and spawn first: an unknown destination must preserve the current road.
    if scene::spawn_road(world, destination).is_none() {
        warn!(?destination, "Unknown road destination");
        return;
    }
    for road in roads {
        // Children is a linked relationship: despawning the root removes the entire road.
        world.despawn(road);
    }
    for (mut transform, mut position, mut velocity, mut angular_velocity, mut hits) in world
        .query_filtered::<(
            &mut Transform,
            &mut Position,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &mut ShapeHits,
        ), With<Player>>()
        .iter_mut(world)
    {
        transform.translation = ROAD_SPAWN.extend(transform.translation.z);
        position.0 = ROAD_SPAWN;
        *velocity = LinearVelocity::ZERO;
        *angular_velocity = AngularVelocity::ZERO;
        hits.clear();
    }
    close_ui(world);
}
