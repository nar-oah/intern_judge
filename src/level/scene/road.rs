use super::{
    building::{factory_scene, residential_scene, shop_scene},
    platform::platform_scene,
    switch_point::switch_point_scene,
};
use crate::{
    level::{RoadId, SceneRoot},
    player::player_scene,
};
use bevy::prelude::*;

pub(in crate::level) const EXAMPLE_ROAD: RoadId = RoadId("example_road");
pub(in crate::level) const MARKET_ROAD: RoadId = RoadId("market_road");
pub(in crate::level) const COURT: RoadId = RoadId("court");

pub(super) fn example_road_scene() -> impl Scene {
    bsn! {
        #ExampleRoad
        SceneRoot
        EXAMPLE_ROAD
        Transform::default()
        Visibility::default()
        Children [
            @player_scene(Vec2::new(-460.0, -130.0))
            --
            @platform_scene(Vec2::new(0.0, -180.0), Vec2::new(700.0, 35.0))
            --
            @platform_scene(Vec2::new(-425.0, -180.0), Vec2::new(150.0, 35.0))
            --
            @platform_scene(Vec2::new(425.0, -180.0), Vec2::new(150.0, 35.0))
            --
            @platform_scene(Vec2::new(-170.0, -96.0), Vec2::new(130.0, 12.0))
            --
            @platform_scene(Vec2::new(170.0, -90.0), Vec2::new(150.0, 20.0))
            --
            @residential_scene(Vec2::new(-310.0, -122.5), Vec2::new(65.0, 80.0))
            --
            @shop_scene(Vec2::new(0.0, -127.5), Vec2::new(70.0, 70.0))
            --
            @factory_scene(Vec2::new(310.0, -117.5), Vec2::new(90.0, 90.0))
            --
            @switch_point_scene(&[MARKET_ROAD, COURT], Vec2::new(460.0, -137.5))
        ]
    }
}

pub(super) fn market_road_scene() -> impl Scene {
    bsn! {
        #MarketRoad
        SceneRoot
        MARKET_ROAD
        Transform::default()
        Visibility::default()
        Children [
            @player_scene(Vec2::new(-460.0, -130.0))
            --
            @platform_scene(Vec2::new(0.0, -180.0), Vec2::new(1000.0, 35.0))
            --
            @shop_scene(Vec2::new(-170.0, -122.5), Vec2::new(100.0, 80.0))
            --
            @residential_scene(Vec2::new(170.0, -112.5), Vec2::new(80.0, 100.0))
            --
            @switch_point_scene(&[COURT], Vec2::new(460.0, -137.5))
        ]
    }
}

pub(super) fn court_scene() -> impl Scene {
    bsn! {
        #Court
        SceneRoot
        COURT
        Transform::default()
        Visibility::default()
        Children [
            @player_scene(Vec2::new(-460.0, -130.0))
            --
            @platform_scene(Vec2::new(0.0, -180.0), Vec2::new(1000.0, 35.0))
        ]
    }
}
