use super::{
    building_scene,
    judge_bench::{CourtDialogue, interact},
};
use bevy::prelude::*;

pub(in crate::level::scene) fn prosecution_scene(position: Vec2) -> impl Scene {
    court_party_scene(
        position,
        CourtDialogue::Prosecution,
        Color::srgb(0.75, 0.3, 0.3),
    )
}

pub(in crate::level::scene) fn defense_scene(position: Vec2) -> impl Scene {
    court_party_scene(
        position,
        CourtDialogue::Defense,
        Color::srgb(0.3, 0.45, 0.8),
    )
}

fn court_party_scene(position: Vec2, dialogue: CourtDialogue, color: Color) -> impl Scene {
    bsn! {
        #CourtParty
        @building_scene(position, Vec2::new(100.0, 70.0), color)
        dialogue
        on(interact)
    }
}
