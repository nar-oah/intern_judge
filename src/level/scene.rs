mod building;
mod platform;
mod road;
mod switch_point;

use super::RoadId;
use bevy::prelude::*;

pub(super) use road::{COURT, EXAMPLE_ROAD, MARKET_ROAD};

pub(super) fn spawn_road(world: &mut World, road: RoadId) -> Option<Entity> {
    let entity = match road {
        EXAMPLE_ROAD => world.spawn_scene(road::example_road_scene()).unwrap(),
        MARKET_ROAD => world.spawn_scene(road::market_road_scene()).unwrap(),
        COURT => world.spawn_scene(road::court_scene()).unwrap(),
        _ => return None,
    };
    Some(entity.id())
}
