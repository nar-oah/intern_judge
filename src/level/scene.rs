mod building;
mod platform;
mod road;
mod switch_point;

use super::RoadId;
use bevy::prelude::*;

// Start is a logical map origin and never has a physical scene.
pub(super) fn road_scene(road: RoadId) -> Option<Box<dyn Scene>> {
    match road {
        RoadId::Start => None,
        RoadId::ExampleRoad => Some(Box::new(road::example_road_scene())),
        RoadId::MarketRoad => Some(Box::new(road::market_road_scene())),
        RoadId::Court => Some(Box::new(road::court_scene())),
    }
}
