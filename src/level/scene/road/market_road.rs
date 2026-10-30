use super::super::{
    building::{convenience_store_scene, tool_store_scene},
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

const FALLING_X: [f32; 4] = [-350.0, -80.0, 80.0, 350.0];

pub(in crate::level::scene) fn market_road_scene(effects: &[RoadEffect]) -> impl Scene + use<> {
    let falling_object = effects.contains(&RoadEffect::FallingObject).then(|| {
        let x = *FALLING_X.choose(&mut rand::rng()).unwrap();
        bsn_list! { @falling_object_scene(Vec2::new(x, 320.0)) }
    });
    let construction_detour = effects.contains(&RoadEffect::ConstructionDetour).then(|| {
        bsn_list! { @construction_detour_scene(Vec2::new(0.0, -132.5), Vec2::splat(60.0)) }
    });

    bsn! {
        #MarketRoad
        SceneRoot
        RoadId::MarketRoad
        Transform::default()
        Visibility::default()
        Children [
            @player_scene(Vec2::new(-460.0, -130.0))
            --
            @platform_scene(Vec2::new(0.0, -180.0), Vec2::new(1000.0, 35.0))
            --
            @tool_store_scene(Vec2::new(-170.0, -122.5), Vec2::new(100.0, 80.0))
            --
            @convenience_store_scene(Vec2::new(170.0, -112.5), Vec2::new(80.0, 100.0), ConvenienceKind::Large)
            --
            @switch_point_scene(Vec2::new(460.0, -137.5))
            --
            {falling_object}
            --
            {construction_detour}
        ]
    }
}
