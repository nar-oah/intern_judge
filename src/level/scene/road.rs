use super::{
    building::{factory_scene, residential_scene, shop_scene},
    platform::platform_scene,
    switch_point::switch_point_scene,
};
use crate::level::RoadId;
use bevy::prelude::*;

/// Positions are local to this root. Despawning it also removes the whole road.
pub(super) fn example_road_scene() -> impl Scene {
    bsn! {
        #ExampleRoad
        RoadId("example_road")
        Transform::default()
        Visibility::default()
        Children [
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
            @switch_point_scene("west_exit", Vec2::new(-460.0, -137.5))
            --
            @switch_point_scene("east_exit", Vec2::new(460.0, -137.5))
        ]
    }
}
