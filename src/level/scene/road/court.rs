use super::super::{
    effect::{construction_detour_scene, falling_object_scene},
    platform::platform_scene,
};
use crate::{
    level::{RoadEffect, RoadId, SceneRoot},
    player::player_scene,
};
use bevy::prelude::*;
use rand::seq::IndexedRandom;

const FALLING_X: [f32; 3] = [-280.0, 0.0, 280.0];

pub(in crate::level::scene) fn court_scene(effects: &[RoadEffect]) -> impl Scene + use<> {
    let falling_object = effects.contains(&RoadEffect::FallingObject).then(|| {
        let x = *FALLING_X.choose(&mut rand::rng()).unwrap();
        bsn_list! { @falling_object_scene(Vec2::new(x, 320.0)) }
    });
    let construction_detour = effects.contains(&RoadEffect::ConstructionDetour).then(|| {
        bsn_list! { @construction_detour_scene(Vec2::new(0.0, -132.5), Vec2::splat(60.0)) }
    });

    bsn! {
        #Court
        SceneRoot
        RoadId::Court
        Transform::default()
        Visibility::default()
        Children [
            @player_scene(Vec2::new(-460.0, -130.0))
            --
            @platform_scene(Vec2::new(0.0, -180.0), Vec2::new(1000.0, 35.0))
            --
            {falling_object}
            --
            {construction_detour}
        ]
    }
}
