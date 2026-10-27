mod building;
mod platform;
mod road;
mod switch_point;

use super::RoadId;
use bevy::prelude::*;

pub(super) use road::{COURT, EXAMPLE_ROAD, MARKET_ROAD};

pub(super) fn road_scene(road: RoadId) -> Box<dyn Scene> {
    match road {
        EXAMPLE_ROAD => Box::new(road::example_road_scene()),
        MARKET_ROAD => Box::new(road::market_road_scene()),
        COURT => Box::new(road::court_scene()),
    }
}
