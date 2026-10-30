mod building;
pub(crate) mod effect;
mod platform;
mod road;
mod switch_point;

use super::{RoadEffect, RoadId};
use bevy::prelude::*;

pub(super) fn road_scene(road: RoadId, effects: &[RoadEffect]) -> Box<dyn Scene> {
    match road {
        RoadId::ExampleRoad => Box::new(road::example_road_scene(effects)),
        RoadId::MarketRoad => Box::new(road::market_road_scene(effects)),
        RoadId::Court => Box::new(road::court_scene(effects)),
    }
}
