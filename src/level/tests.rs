use super::{
    interaction::{DemoWallet, Interactable},
    platform::Platform,
    road::RoadId,
};
use crate::player::{Player, PlayerPlugin, SpeedBoost};
use avian2d::prelude::*;
use bevy::{
    asset::AssetPlugin, input::InputPlugin, prelude::*, scene::ScenePlugin,
    time::TimeUpdateStrategy,
};
use leafwing_input_manager::user_input::Buttonlike;
use std::time::Duration;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        AssetPlugin::default(),
        ScenePlugin,
        InputPlugin,
        PhysicsPlugins::default().with_length_unit(20.0),
        PlayerPlugin,
        super::LevelPlugin,
    ))
    .insert_resource(Gravity(Vec2::NEG_Y * 1000.0))
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_micros(
        15_625,
    )));
    app.finish();
    app.cleanup();
    app.update();
    app
}

fn player_entity(app: &mut App) -> Entity {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<Player>>()
        .single(world)
        .unwrap()
}

#[test]
fn bsn_road_spawns_platforms_and_interactions() {
    let mut app = test_app();
    let world = app.world_mut();

    let mut roads = world.query::<&RoadId>();
    assert_eq!(
        roads.iter(world).map(|id| id.0).collect::<Vec<_>>(),
        vec![1]
    );

    let mut platforms = world.query_filtered::<Entity, With<Platform>>();
    assert_eq!(platforms.iter(world).count(), 2);

    let mut interactions = world.query::<&Interactable>();
    assert_eq!(interactions.iter(world).count(), 4);
}

#[test]
fn store_interaction_grants_a_temporary_speed_boost() {
    let mut app = test_app();
    let player = player_entity(&mut app);

    // The player starts close enough to the convenience store in road_01.
    KeyCode::KeyE.press(app.world_mut());
    app.update();

    assert!(app.world().get::<SpeedBoost>(player).is_some());
}

#[test]
fn restaurant_can_only_pay_once_in_this_prototype() {
    let mut app = test_app();
    let player = player_entity(&mut app);

    app.world_mut()
        .entity_mut(player)
        .get_mut::<Transform>()
        .unwrap()
        .translation = Vec3::new(-220.0, -100.0, 0.0);
    app.update();

    KeyCode::KeyE.press(app.world_mut());
    app.update();
    assert_eq!(app.world().resource::<DemoWallet>().0, 10);

    KeyCode::KeyE.release(app.world_mut());
    app.update();
    KeyCode::KeyE.press(app.world_mut());
    app.update();
    assert_eq!(app.world().resource::<DemoWallet>().0, 10);
}
