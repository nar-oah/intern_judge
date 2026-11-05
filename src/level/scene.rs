mod building;
pub(crate) mod effect;
pub(super) mod ending;
mod platform;
mod road;
mod switch_point;

use super::{CurrentCase, RoadEffect, RoadEffects, RoadId, TodayRoads, case};
use crate::config::CaseDefs;
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

pub(super) fn enter_road(world: &mut World, road: RoadId) -> Box<dyn Scene> {
    let (case_road, dialogue_prefix) = if road == RoadId::Court {
        let selected = case::select_case(
            world.resource::<TodayRoads>(),
            world.resource::<RoadEffects>(),
            world.resource::<CaseDefs>(),
            &mut rand::rng(),
        );
        let prefix = selected.case.dialogue_prefix.clone();
        let case_road = selected.road;
        world.resource_mut::<CurrentCase>().0 = Some(selected);
        (case_road, prefix)
    } else {
        world.resource_mut::<TodayRoads>().0.insert(road);
        (road, String::new())
    };
    let effects = world.resource::<RoadEffects>().0.get(&road);
    road_scene(road, effects.map_or(&[], Vec::as_slice), &dialogue_prefix, case_road)
}

pub(super) fn road_scene(
    road: RoadId,
    effects: &[RoadEffect],
    dialogue_prefix: &str,
    case_road: RoadId,
) -> Box<dyn Scene> {
    match road {
        RoadId::ExampleRoad => Box::new(road::example_road_scene(effects)),
        RoadId::MarketRoad => Box::new(road::market_road_scene(effects)),
        RoadId::Court => Box::new(road::court_scene(effects, dialogue_prefix, case_road)),
        RoadId::EmptyRoad1
        | RoadId::EmptyRoad2
        | RoadId::EmptyRoad3
        | RoadId::EmptyRoad4
        | RoadId::EmptyRoad5
        | RoadId::EmptyRoad6
        | RoadId::EmptyRoad7 => Box::new(road::empty_road_scene(road)),
    }
}
