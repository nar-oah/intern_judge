use super::{COURT, EXAMPLE_ROAD, NEXT_ROAD, ROAD_SPAWN, RoadId, scene};
use crate::player::Player;
use avian2d::prelude::*;
use bevy::prelude::*;
use std::time::Duration;

#[derive(Resource, Default)]
pub(crate) enum TravelUi {
    #[default]
    Closed,
    Choosing { destinations: &'static [RoadId] },
    Court,
}

impl TravelUi {
    pub(crate) fn is_open(&self) -> bool {
        !matches!(self, Self::Closed)
    }
}

pub(super) fn open_destinations(world: &mut World, destinations: &'static [RoadId]) {
    if world.resource::<TravelUi>().is_open() || destinations.is_empty() {
        return;
    }
    *world.resource_mut::<TravelUi>() = TravelUi::Choosing { destinations };
    for mut velocity in world
        .query_filtered::<&mut LinearVelocity, With<Player>>()
        .iter_mut(world)
    {
        *velocity = LinearVelocity::ZERO;
    }
    let mut time = world.resource_mut::<Time<Physics>>();
    time.pause();
    // Avian reuses the last delta on the first paused step unless it is cleared.
    time.advance_by(Duration::ZERO);
}

pub(super) fn close_ui(world: &mut World) {
    if world.resource::<TravelUi>().is_open() {
        *world.resource_mut::<TravelUi>() = TravelUi::Closed;
        world.resource_mut::<Time<Physics>>().unpause();
    }
}

pub(super) fn select_destination(world: &mut World, destination: RoadId) {
    let TravelUi::Choosing { destinations } = world.resource::<TravelUi>() else {
        return;
    };
    if !destinations.contains(&destination) {
        return;
    }
    if destination == COURT {
        *world.resource_mut::<TravelUi>() = TravelUi::Court;
        return;
    }
    // Validate before removing the current scene. Unknown IDs leave the menu open.
    if !matches!(destination, EXAMPLE_ROAD | NEXT_ROAD) {
        warn!(?destination, "Unknown road destination");
        return;
    }

    let roads: Vec<_> = world
        .query_filtered::<Entity, With<RoadId>>()
        .iter(world)
        .collect();
    for road in roads {
        // Children is a linked relationship: despawning the root removes the entire road.
        world.despawn(road);
    }
    match destination {
        EXAMPLE_ROAD => world.spawn_scene(scene::example_road_scene()).unwrap(),
        NEXT_ROAD => world.spawn_scene(scene::market_road_scene()).unwrap(),
        _ => unreachable!(),
    };

    for (mut transform, mut position, mut velocity, mut hits) in world
        .query_filtered::<
            (&mut Transform, &mut Position, &mut LinearVelocity, &mut ShapeHits),
            With<Player>,
        >()
        .iter_mut(world)
    {
        transform.translation = ROAD_SPAWN.extend(transform.translation.z);
        position.0 = ROAD_SPAWN;
        *velocity = LinearVelocity::ZERO;
        hits.clear();
    }
    close_ui(world);
}
