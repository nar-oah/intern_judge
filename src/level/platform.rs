use avian2d::prelude::*;
use bevy::prelude::*;

// The level chooses a logical size. Only this scene decides how to render it.
// Later we can replace the colored Sprite with tiled / sliced artwork.
#[derive(Component, Clone, Default)]
pub(super) struct Platform;

pub(super) fn platform(size: Vec2) -> impl Scene {
    bsn! {
        #Platform
        Platform
        Sprite {
            color: Color::srgb(0.65, 0.68, 0.75),
            custom_size: Option::Some(size),
        }
        RigidBody::Static
        Collider::rectangle(size.x, size.y)
    }
}
