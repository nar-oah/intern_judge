use avian2d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::InputMap;

use crate::config::PlayerConfig;
use crate::player::{Player, PlayerAction};

/// A code-driven BSN scene (like a small Godot .tscn).
///
/// Numbers come from player.ron, while BSN describes which ECS components
/// the player has. Avian components without a BSN default are constructed
/// using ad-hoc templates.
pub(crate) fn player_scene(config: &PlayerConfig, input_map: InputMap<PlayerAction>) -> impl Scene {
    let size = Vec2::new(config.size[0], config.size[1]);
    let move_speed = config.move_speed;
    let jump_speed = config.jump_speed;

    bsn! {
        #Player
        Player {
            move_speed: move_speed,
            jump_speed: jump_speed,
        }
        Sprite {
            color: {Color::srgb(0.3, 0.7, 0.9)},
            custom_size: {Some(size)},
        }
        Transform {
            translation: {Vec3::new(0.0, -80.0, 0.0)},
        }
        template(|_| Ok(RigidBody::Dynamic))
        template(move |_| Ok(Collider::rectangle(size.x, size.y)))
        template(|_| Ok(LinearVelocity::ZERO))
        template(|_| Ok(LockedAxes::ROTATION_LOCKED))
        template(|_| Ok(Friction::ZERO))
        template(|_| Ok(Restitution::ZERO))
        template(move |_| {
            Ok(
                ShapeCaster::new(
                    Collider::rectangle(size.x - 2.0, size.y - 2.0),
                    Vec2::ZERO,
                    0.0,
                    Dir2::NEG_Y,
                )
                .with_max_distance(6.0),
            )
        })
        template(move |_| Ok(input_map.clone()))
    }
}
