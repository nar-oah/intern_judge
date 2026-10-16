mod level;
mod player;

use avian2d::prelude::*;
use bevy::prelude::*;

use level::LevelPlugin;
use player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default().with_length_unit(20.0),
            PlayerPlugin,
            LevelPlugin,
        ))
        .insert_resource(Gravity(Vec2::NEG_Y * 1000.0))
        .insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.13)))
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
