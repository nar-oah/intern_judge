use super::super::{
    building::{cat_scene, convenience_store_scene, lottery_store_scene, tool_store_scene},
    effect::{construction_detour_scene, falling_object_scene},
    platform::platform_scene,
    switch_point::switch_point_scene,
};
use crate::{
    config::ConvenienceKind,
    level::{RoadEffect, RoadId, SceneRoot},
    player::player_scene,
};
use bevy::prelude::*;
use rand::seq::IndexedRandom;

const FALLING_X: [f32; 4] = [-420.0, -250.0, 90.0, 250.0];

pub(super) fn example_road_scene(effects: &[RoadEffect]) -> impl Scene {
    let falling_object = effects.contains(&RoadEffect::FallingObject).then(|| {
        let x = *FALLING_X.choose(&mut rand::rng()).unwrap();
        falling_object_scene(Vec2::new(x, 320.0))
    });
    let construction_detour = effects
        .contains(&RoadEffect::ConstructionDetour)
        .then(|| construction_detour_scene(Vec2::new(90.0, -132.5), Vec2::splat(60.0)));

    bsn! {
        #ExampleRoad
        SceneRoot
        RoadId::ExampleRoad
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
            @convenience_store_scene(Vec2::new(-310.0, -122.5), Vec2::new(65.0, 80.0), ConvenienceKind::Small)
            --
            @tool_store_scene(Vec2::new(0.0, -127.5), Vec2::new(70.0, 70.0))
            --
            @lottery_store_scene(Vec2::new(310.0, -117.5), Vec2::new(90.0, 90.0))
            --
            @cat_scene(Vec2::new(170.0, -66.0), Vec2::new(28.0, 28.0))
            --
            @switch_point_scene(Vec2::new(460.0, -137.5))
            --
            @falling_object
            --
            @construction_detour
        ]
    }
}
