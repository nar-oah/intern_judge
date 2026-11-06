mod building;
pub(crate) mod effect;
pub(super) mod ending;
mod platform;
mod road;
mod switch_point;

use super::{CurrentCase, RoadEffect, RoadEffects, RoadId, SceneRoot, TodayRoads, case};
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

#[derive(Event)]
pub(super) struct EnterRoad(pub RoadId);

pub(super) fn enter_road(
    request: On<EnterRoad>,
    mut commands: Commands,
    roots: Query<Entity, With<SceneRoot>>,
    mut today: ResMut<TodayRoads>,
    effects: Res<RoadEffects>,
    cases: Res<CaseDefs>,
    mut current: ResMut<CurrentCase>,
) {
    let road = request.0;
    if road == RoadId::Court {
        current.0 = Some(case::select_case(
            &today,
            &effects,
            &cases,
            &mut rand::rng(),
        ));
    } else {
        today.0.insert(road);
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.spawn_scene(road_scene(
        road,
        effects.0.get(&road).map_or(&[], Vec::as_slice),
    ));
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
