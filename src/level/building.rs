use bevy::prelude::*;

use super::interaction::Interactable;

// The buildings own their appearance and interaction kind, not their position.
// Each road specifies a Transform when including these scenes.
pub(super) fn convenience_store() -> impl Scene {
    bsn! {
        #ConvenienceStore
        Interactable::ConvenienceStore
        Sprite::from_color(Color::srgb(0.32, 0.72, 0.45), Vec2::new(70.0, 90.0))
    }
}

pub(super) fn restaurant() -> impl Scene {
    bsn! {
        #Restaurant
        Interactable::Restaurant { worked: false }
        Sprite::from_color(Color::srgb(0.85, 0.43, 0.31), Vec2::new(70.0, 90.0))
    }
}
