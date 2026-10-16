use avian2d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 50.0);

#[derive(Component)]
struct Player {
    move_speed: f32,
    jump_speed: f32,
}

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
            .add_systems(Startup, spawn_player)
            .add_systems(FixedUpdate, player_movement);
    }
}

fn spawn_player(mut commands: Commands) {
    let input_map = InputMap::new([
        (PlayerAction::Left, KeyCode::KeyA),
        (PlayerAction::Right, KeyCode::KeyD),
        (PlayerAction::Jump, KeyCode::Space),
    ]);

    commands.spawn((
        Player {
            move_speed: 220.0,
            jump_speed: 420.0,
        },
        Sprite::from_color(Color::srgb(0.3, 0.7, 0.9), PLAYER_SIZE),
        Transform::from_xyz(0.0, -80.0, 0.0),
        RigidBody::Dynamic,
        Collider::rectangle(PLAYER_SIZE.x, PLAYER_SIZE.y),
        LinearVelocity::ZERO,
        LockedAxes::ROTATION_LOCKED,
        Friction::ZERO,
        Restitution::ZERO,
        // The slightly smaller shape probes downward for a walkable surface.
        ShapeCaster::new(
            Collider::rectangle(PLAYER_SIZE.x - 2.0, PLAYER_SIZE.y - 2.0),
            Vec2::ZERO,
            0.0,
            Dir2::NEG_Y,
        )
        .with_max_distance(6.0),
        input_map,
    ));
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

        // Avian updates ShapeHits during physics; this reads the last step's results.
        let grounded = hits
            .iter()
            .any(|hit| (-hit.normal2).dot(Vec2::Y) > 0.7);

        if grounded && actions.just_pressed(&PlayerAction::Jump) {
            velocity.y = player.jump_speed;
        }
    }
}
