use super::super::{
    building::{BuildingDialogue, dialogue_building_scene},
    effect::{construction_detour_scene, falling_object_scene},
    platform::platform_scene,
};
use crate::{
    level::{RoadEffect, RoadId, SceneRoot},
    player::player_scene,
};
use bevy::{prelude::*, text::FontSourceTemplate};
use rand::seq::IndexedRandom;

const FALLING_X: [f32; 3] = [-280.0, 0.0, 280.0];

pub(in crate::level::scene) fn case_hud_scene(road: RoadId) -> impl Scene {
    bsn! {
        #CaseHud
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: percent(50),
            padding: UiRect::all(px(12)),
            border_radius: BorderRadius::all(px(6)),
        }
        UiTransform::from_translation(Val2::percent(-50.0, 0.0))
        GlobalZIndex(200)
        BackgroundColor(Color::srgb(0.12, 0.14, 0.2))
        Pickable::IGNORE
        Children [
            Text::new(format!("今日涉案路段：{road:?}"))
            TextFont {
                font: FontSourceTemplate::Handle("fonts/court-dialogue.ttf"),
                font_size: FontSize::Px(22.0),
            }
            TextColor(Color::srgb(0.95, 0.96, 1.0))
            Pickable::IGNORE
        ]
    }
}

pub(in crate::level::scene) fn court_scene(
    effects: &[RoadEffect],
    dialogue_prefix: &str,
) -> impl Scene + use<> {
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
            Name::new("Prosecution")
            @dialogue_building_scene(
                Vec2::new(-220.0, -127.5),
                Vec2::new(80.0, 70.0),
                Color::srgb(0.8, 0.3, 0.3),
                BuildingDialogue::new("dialogue/court.rep", format!("{dialogue_prefix}_prosecution")),
            )
            --
            Name::new("JudgeBench")
            @dialogue_building_scene(
                Vec2::new(0.0, -127.5),
                Vec2::new(120.0, 70.0),
                Color::srgb(0.65, 0.45, 0.2),
                BuildingDialogue::new("dialogue/court.rep", "verdict"),
            )
            --
            Name::new("Defense")
            @dialogue_building_scene(
                Vec2::new(220.0, -127.5),
                Vec2::new(80.0, 70.0),
                Color::srgb(0.3, 0.45, 0.8),
                BuildingDialogue::new("dialogue/court.rep", format!("{dialogue_prefix}_defense")),
            )
            --
            {falling_object}
            --
            {construction_detour}
        ]
    }
}
