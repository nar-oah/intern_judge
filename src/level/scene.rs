mod building;
pub(crate) mod effect;
pub(super) mod ending;
mod platform;
mod road;
mod switch_point;

use super::{RoadEffect, RoadId};
use bevy::prelude::*;

pub(super) fn replace_scene(world: &mut World, root: Entity, next: impl Scene) {
    world.despawn(root);
    world.spawn_scene(next).unwrap();
}

pub(super) fn road_scene(road: RoadId, effects: &[RoadEffect]) -> Box<dyn Scene> {
    match road {
        RoadId::ExampleRoad => Box::new(road::example_road_scene(effects)),
        RoadId::MarketRoad => Box::new(road::market_road_scene(effects)),
        RoadId::Court => Box::new(road::court_scene(effects)),
    }
}
