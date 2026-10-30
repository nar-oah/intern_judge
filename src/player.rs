mod scene;
#[cfg(test)]
mod tests;
use crate::{
    config::{GameSettings, GameState},
    game::Stamina,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
pub(crate) use scene::player_scene;

#[derive(Component, Clone, Default)]
pub(crate) struct Player {
    move_speed: f32,
    jump_speed: f32,
}

#[derive(Component)]
pub(crate) struct Stunned {
    pub remaining: f32,
}

#[derive(Actionlike, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub(crate) enum PlayerAction {
    Left,
    Right,
    Sprint,
    Jump,
    Interact,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(
                FixedUpdate,
                (tick_stun, player_movement)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn tick_stun(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut players: Query<(Entity, &mut Stunned), With<Player>>,
) {
    for (entity, mut stunned) in &mut players {
        stunned.remaining -= time.delta_secs();
        if stunned.remaining <= 0.0 {
            commands.entity(entity).remove::<Stunned>();
        }
    }
}

fn player_movement(
    time: Res<Time<Fixed>>,
    settings: Res<GameSettings>,
    mut stamina: ResMut<Stamina>,
    mut players: Query<
        (
            &Player,
            &ActionState<PlayerAction>,
            &ShapeHits,
            &mut LinearVelocity,
        ),
        Without<Stunned>,
    >,
) {
    for (player, actions, hits, mut velocity) in &mut players {
        let left = actions.pressed(&PlayerAction::Left);
        let right = actions.pressed(&PlayerAction::Right);
        let direction = i8::from(right) - i8::from(left);
        let sprinting = direction != 0 && actions.pressed(&PlayerAction::Sprint) && stamina.0 > 0.0;
        let speed_multiplier = if sprinting { settings.multiplier } else { 1.0 };
        velocity.x = f32::from(direction) * player.move_speed * speed_multiplier;
        if sprinting {
            stamina.0 = (stamina.0 - settings.per_second * time.delta_secs()).max(0.0);
        }
        let grounded = hits.iter().any(|hit| (-hit.normal2).dot(Vec2::Y) > 0.7);
        if grounded && actions.just_pressed(&PlayerAction::Jump) {
            velocity.y = player.jump_speed;
        }
    }
}
