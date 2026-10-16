use avian2d::prelude::*;
use bevy::prelude::*;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_test_platforms);
    }
}

fn spawn_test_platforms(mut commands: Commands) {
    spawn_platform(
        &mut commands,
        Vec2::new(0.0, -180.0),
        Vec2::new(700.0, 35.0),
    );
    spawn_platform(
        &mut commands,
        Vec2::new(170.0, -90.0),
        Vec2::new(150.0, 20.0),
    );
}

fn spawn_platform(commands: &mut Commands, position: Vec2, size: Vec2) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.65, 0.68, 0.75), size),
        Transform::from_xyz(position.x, position.y, 0.0),
        RigidBody::Static,
        Collider::rectangle(size.x, size.y),
    ));
}
