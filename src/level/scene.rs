mod building;
pub(crate) mod effect;
pub(super) mod ending;
mod platform;
mod road;
mod switch_point;

use super::{RoadEffect, RoadId};
use bevy::prelude::*;

pub(super) fn replace_scene(
    world: &mut World,
    roots: impl IntoIterator<Item = Entity>,
    next: impl Scene,
) {
    for root in roots {
        world.despawn(root);
    }
    world.spawn_scene(next).unwrap();
}

pub(super) fn road_scene(road: RoadId, effects: &[RoadEffect]) -> Box<dyn Scene> {
    match road {
        RoadId::ExampleRoad => Box::new(road::example_road_scene(effects)),
        RoadId::MarketRoad => Box::new(road::market_road_scene(effects)),
        RoadId::Court => Box::new(road::court_scene(effects)),
        RoadId::EmptyRoad1
        | RoadId::EmptyRoad2
        | RoadId::EmptyRoad3
        | RoadId::EmptyRoad4
        | RoadId::EmptyRoad5
        | RoadId::EmptyRoad6
        | RoadId::EmptyRoad7 => Box::new(road::empty_road_scene(road)),
    }
}
