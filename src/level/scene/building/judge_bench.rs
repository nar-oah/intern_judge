use super::building_scene;
use crate::level::dialogue::{CourtDialogue, start_court_dialogue};
use bevy::prelude::*;

pub(in crate::level::scene) fn judge_bench_scene(position: Vec2) -> impl Scene {
    bsn! {
        #JudgeBench
        CourtDialogue("verdict")
        @building_scene(
            position,
            Vec2::new(120.0, 70.0),
            Color::srgb(0.65, 0.45, 0.2),
        )
        on(start_court_dialogue)
    }
}
