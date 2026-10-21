use crate::level::Platform;
use crate::physics::GameLayer;
use avian2d::prelude::*;
use bevy::prelude::*;

pub(super) fn platform_scene(position: Vec2, size: Vec2) -> impl Scene {
    let platform = Platform { size };
    let size = platform.size;
    bsn! {
        platform
        Sprite {
            color: Color::srgb(0.65, 0.68, 0.75),
            custom_size: Option::Some(size),
        }
        Transform::from_xyz(position.x, position.y, 0.0)
        RigidBody::Static
        Collider::rectangle(size.x, size.y)
        CollisionLayers::new(GameLayer::Ground, LayerMask::ALL)
    }
}
