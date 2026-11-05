mod court;
mod example_road;
mod market_road;
#[cfg(test)]
mod tests;

pub(super) use court::court_scene;
pub(super) use example_road::example_road_scene;
pub(super) use market_road::market_road_scene;

use super::{platform::platform_scene, switch_point::switch_point_scene};
use crate::{
    level::{RoadId, SceneRoot},
    player::player_scene,
};
use bevy::prelude::*;

pub(super) fn empty_road_scene(road: RoadId) -> impl Scene {
    bsn! {
        #EmptyRoad
        SceneRoot
        road
        Transform::default()
        Visibility::default()
        Children [
            @player_scene(Vec2::new(-460.0, -130.0))
            --
            @platform_scene(Vec2::new(0.0, -180.0), Vec2::new(1000.0, 35.0))
            --
            @switch_point_scene(Vec2::new(460.0, -137.5))
        ]
    }
}
