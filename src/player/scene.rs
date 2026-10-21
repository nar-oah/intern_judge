use super::{Player, PlayerAction};
use crate::physics::GameLayer;
use avian2d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 50.0);

pub(super) fn player_scene() -> impl Scene {
    let input_map = InputMap::new([
        (PlayerAction::Left, KeyCode::KeyA),
        (PlayerAction::Right, KeyCode::KeyD),
        (PlayerAction::Jump, KeyCode::Space),
        (PlayerAction::Interact, KeyCode::KeyE),
    ]);

    bsn! {
        Player {
            move_speed: 220.0,
            jump_speed: 420.0,
        }
        Sprite {
            color: Color::srgb(0.3, 0.7, 0.9),
            custom_size: Option::Some(PLAYER_SIZE),
        }
        Transform::from_xyz(0.0, -80.0, 0.0)
        RigidBody::Dynamic
        Collider::rectangle(PLAYER_SIZE.x, PLAYER_SIZE.y)
        LinearVelocity::ZERO
        LockedAxes::ROTATION_LOCKED
        Friction::ZERO
        Restitution::ZERO
        ShapeCaster::new(
            Collider::rectangle(PLAYER_SIZE.x - 2.0, PLAYER_SIZE.y - 2.0),
            Vec2::ZERO,
            0.0,
            Dir2::NEG_Y,
        )
        .with_max_distance(6.0)
        .with_query_filter(SpatialQueryFilter::from_mask(GameLayer::Ground))
        input_map
    }
}
