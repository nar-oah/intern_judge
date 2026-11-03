use super::building_scene;
use crate::level::dialogue::{CourtDialogue, start_court_dialogue};
use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub(in crate::level::scene) enum CourtParty {
    Prosecution,
    Defense,
}

pub(in crate::level::scene) fn court_party_scene(position: Vec2, party: CourtParty) -> impl Scene {
    let (name, node, color) = match party {
        CourtParty::Prosecution => ("Prosecution", "prosecution", Color::srgb(0.8, 0.3, 0.3)),
        CourtParty::Defense => ("Defense", "defense", Color::srgb(0.3, 0.45, 0.8)),
    };
    bsn! {
        Name::new(name)
        party
        CourtDialogue(node)
        @building_scene(position, Vec2::new(80.0, 70.0), color)
        on(start_court_dialogue)
    }
}
