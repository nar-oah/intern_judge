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

// The prototype store applies this component; movement reads it directly.
#[derive(Component)]
pub(crate) struct SpeedBoost(pub Timer);

#[derive(Actionlike, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
enum PlayerAction {
    Left,
    Right,
    Jump,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(Startup, scene::player_scene.spawn())
            .add_systems(FixedUpdate, player_movement)
            .add_systems(Update, tick_speed_boost);
    }
}

fn tick_speed_boost(
    mut commands: Commands,
    time: Res<Time>,
    mut boosts: Query<(Entity, &mut SpeedBoost)>,
) {
    for (entity, mut boost) in &mut boosts {
        if boost.0.tick(time.delta()).is_finished() {
            commands.entity(entity).remove::<SpeedBoost>();
        }
    }
}

type PlayerMovementComponents<'a> = (
    &'a Player,
    &'a ActionState<PlayerAction>,
    &'a ShapeHits,
    Option<&'a SpeedBoost>,
    &'a mut LinearVelocity,
);

fn player_movement(mut players: Query<PlayerMovementComponents<'_>>) {
    for (player, actions, hits, speed_boost, mut velocity) in &mut players {
        let left = actions.pressed(&PlayerAction::Left);
        let right = actions.pressed(&PlayerAction::Right);
        let direction = i8::from(right) - i8::from(left);
        let multiplier = if speed_boost.is_some() { 1.5 } else { 1.0 };
        velocity.x = f32::from(direction) * player.move_speed * multiplier;
        let grounded = hits.iter().any(|hit| (-hit.normal2).dot(Vec2::Y) > 0.7);
        if grounded && actions.just_pressed(&PlayerAction::Jump) {
            velocity.y = player.jump_speed;
        }
    }
}
