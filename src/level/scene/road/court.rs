use super::super::{
    effect::{construction_detour_scene, falling_object_scene},
    platform::platform_scene,
};
use crate::{
    level::{
        Interactable, RoadEffect, RoadId, SceneRoot,
        court::{CourtHearing, CourtNpc, begin_dialogue},
    },
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
        CourtHearing::default()
        Transform::default()
        Visibility::default()
        Children [
            @player_scene(Vec2::new(0.0, -130.0))
            --
            @platform_scene(Vec2::new(0.0, -180.0), Vec2::new(1000.0, 35.0))
            --
            @npc_scene(CourtNpc::Prosecution, -260.0, Color::srgb(0.8, 0.3, 0.25))
            --
            @npc_scene(CourtNpc::Defense, 260.0, Color::srgb(0.3, 0.7, 0.4))
            --
            {falling_object}
            --
            {construction_detour}
        ]
    }
}

fn npc_scene(npc: CourtNpc, x: f32, color: Color) -> impl Scene {
    let name = match npc {
        CourtNpc::Prosecution => "公诉方",
        CourtNpc::Defense => "辩护方",
    };
    let font = bevy::text::FontSourceTemplate::Handle("fonts/CourtDialogueSans.otf".into());
    bsn! {
        Name::new(name)
        npc
        Interactable
        Transform::from_xyz(x, -132.5, -1.0)
        Sprite {
            color,
            custom_size: Option::Some(Vec2::new(40.0, 60.0)),
        }
        Collider::rectangle(120.0, 100.0)
        Sensor
        CollidingEntities::default()
        on(begin_dialogue)
        Children [
            Text2d::new(format!("{name}\n靠近按 E 对话"))
            TextFont {
                font,
                font_size: FontSize::Px(18.0),
            }
            TextColor(Color::srgb(0.95, 0.96, 1.0))
            Transform::from_xyz(0.0, 65.0, 1.0)
        ]
    }
}
