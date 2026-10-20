use bevy::prelude::*;

use super::interaction::Interactable;

// Both exits behave identically for now. No destination or scene loading yet.
pub(super) fn transition_point() -> impl Scene {
    bsn! {
        #TransitionPoint
        Interactable::TransitionPoint
        Sprite::from_color(Color::srgb(0.94, 0.79, 0.25), Vec2::new(20.0, 60.0))
    }
}
