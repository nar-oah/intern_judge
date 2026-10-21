mod scene;
#[cfg(test)]
mod tests;
use avian2d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Component, Clone, Default)]
pub(crate) struct Player {
    move_speed: f32,
    jump_speed: f32,
}

#[derive(Actionlike, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub(crate) enum PlayerAction {
    Left,
    Right,
    Jump,
    Interact,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(Startup, scene::player_scene.spawn())
            .add_systems(FixedUpdate, player_movement);
    }
}

fn player_movement(
    mut players: Query<(
        &Player,
        &ActionState<PlayerAction>,
        &ShapeHits,
        &mut LinearVelocity,
    )>,
) {
    for (player, actions, hits, mut velocity) in &mut players {
        let left = actions.pressed(&PlayerAction::Left);
        let right = actions.pressed(&PlayerAction::Right);
        let direction = i8::from(right) - i8::from(left);
        velocity.x = f32::from(direction) * player.move_speed;
        let grounded = hits.iter().any(|hit| (-hit.normal2).dot(Vec2::Y) > 0.7);
        if grounded && actions.just_pressed(&PlayerAction::Jump) {
            velocity.y = player.jump_speed;
        }
    }
}
