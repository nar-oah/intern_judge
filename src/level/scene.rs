mod building;
mod platform;
mod road;
mod switch_point;

use super::RoadId;
use bevy::prelude::*;

pub(super) fn road_scene(road: RoadId) -> Box<dyn Scene> {
    match road {
        RoadId::ExampleRoad => Box::new(road::example_road_scene()),
        RoadId::MarketRoad => Box::new(road::market_road_scene()),
        RoadId::Court => Box::new(road::court_scene()),
    }
}
