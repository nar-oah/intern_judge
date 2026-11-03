use super::super::{
    effect::{construction_detour_scene, falling_object_scene},
    platform::platform_scene,
};
use crate::{
    level::{Interactable, RoadEffect, RoadId, SceneRoot, dialogue::open_dialogue},
    player::player_scene,
};
use avian2d::prelude::*;
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
            @judge_bench_scene()
            --
            {falling_object}
            --
            {construction_detour}
        ]
    }
}

fn judge_bench_scene() -> impl Scene {
    let size = Vec2::new(120.0, 70.0);
    bsn! {
        #JudgeBench
        Interactable
        Sprite {
            color: Color::srgb(0.65, 0.45, 0.2),
            custom_size: Option::Some(size),
        }
        Transform::from_xyz(0.0, -127.5, -1.0)
        Collider::rectangle(size.x, size.y)
        Sensor
        CollidingEntities::default()
        on(open_dialogue)
    }
}
