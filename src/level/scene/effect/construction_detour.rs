use super::ConstructionDetour;
use crate::physics::GameLayer;
use avian2d::prelude::*;
use bevy::prelude::*;

pub(in crate::level::scene) fn construction_detour_scene(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        ConstructionDetour
        Sprite {
            color: Color::srgb(1.0, 0.55, 0.05),
            custom_size: Option::Some(size),
        }
        Transform::from_translation(position.extend(0.0))
        RigidBody::Static
        Collider::rectangle(size.x, size.y)
        CollisionLayers::new(GameLayer::Ground, LayerMask::ALL)
    }
}
