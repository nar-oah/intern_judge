use crate::{
    config::{GameState, RoadGraph},
    game::CommuteState,
    level::{
        CurrentCase, InitialRoad, LevelPlugin, RoadEffect, RoadEffects, RoadId, RoadUnlocks, SceneRoot, SwitchPoint, TodayRoads,
        navigation::open_destinations,
        scene::{self, effect::{ConstructionDetour, FallingObject}},
        tests::support::{activate, assert_commute, assert_road, assert_today_roads},
    },
    player::Player,
    test_support::{assert_absent, assert_removed, finish_app, level_app, scene_entities, single},
};
use bevy::prelude::*;

#[test]
fn new_game_initializes_road_progress_and_clears_the_previous_games_changes() {
    let mut app = level_app();
    app.insert_state(GameState::Loading)
        .add_plugins(LevelPlugin);
    finish_app(&mut app);
    app.world_mut()
        .resource_mut::<RoadGraph>()
        .initial_unlocks
        .insert(RoadId::MarketRoad, false);
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    assert!(app.world().resource::<RoadEffects>().0.is_empty());
    assert!(!app.world().resource::<RoadUnlocks>().0[&RoadId::MarketRoad]);
    let (root, _) = assert_road(&mut app, RoadId::ExampleRoad);
    assert_today_roads(&app, &[RoadId::ExampleRoad]);
    assert!(app.world().resource::<CurrentCase>().0.is_none());
    assert_absent::<FallingObject>(&mut app);
    assert_absent::<ConstructionDetour>(&mut app);

    app.world_mut()
        .resource_mut::<RoadEffects>()
        .0
        .insert(RoadId::Court, vec![RoadEffect::ConstructionDetour]);
    app.world_mut()
        .resource_mut::<RoadUnlocks>()
        .0
        .insert(RoadId::MarketRoad, true);
    // Ending the old session unloads its scene before the next start.
    app.world_mut().despawn(root);
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::MainMenu);
    app.update();
    assert_eq!(
        app.world().resource::<RoadEffects>().0[&RoadId::Court],
        [RoadEffect::ConstructionDetour]
    );
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    assert!(app.world().resource::<RoadEffects>().0.is_empty());
    assert!(!app.world().resource::<RoadUnlocks>().0[&RoadId::MarketRoad]);
    assert_road(&mut app, RoadId::ExampleRoad);
    assert_absent::<FallingObject>(&mut app);
    assert_absent::<ConstructionDetour>(&mut app);
}

#[test]
fn road_effects_survive_travel_and_spawn_new_children_on_each_visit() {
    let mut app = level_app();
    app.add_plugins(LevelPlugin);
    finish_app(&mut app);
    app.world_mut().resource_mut::<RoadEffects>().0 = [
        (RoadId::ExampleRoad, vec![RoadEffect::FallingObject]),
        (
            RoadId::MarketRoad,
            vec![RoadEffect::ConstructionDetour, RoadEffect::FallingObject],
        ),
    ]
    .into();
    let root = single::<SceneRoot>(&mut app);
    app.world_mut().despawn(root);
    app.world_mut()
        .spawn_scene(scene::road_scene(
            RoadId::ExampleRoad,
            &[RoadEffect::FallingObject],
            "",
            RoadId::ExampleRoad,
        ))
        .unwrap();
    // A test graph permits revisiting roads to check per-entry scene creation.
    app.world_mut()
        .resource_mut::<RoadGraph>()
        .roads
        .insert(RoadId::MarketRoad, vec![RoadId::ExampleRoad]);
    let effects = app.world().resource::<RoadEffects>().0.clone();
    let initial_falling_object = single::<FallingObject>(&mut app);
    let initial_root = single::<SceneRoot>(&mut app);
    assert_eq!(
        app.world()
            .get::<ChildOf>(initial_falling_object)
            .unwrap()
            .parent(),
        initial_root
    );
    assert_absent::<ConstructionDetour>(&mut app);
    assert_today_roads(&app, &[RoadId::ExampleRoad]);

    for destination in [RoadId::MarketRoad, RoadId::ExampleRoad, RoadId::MarketRoad] {
        let old_root = single::<SceneRoot>(&mut app);
        let old_scene = scene_entities(app.world(), old_root);
        let player = single::<Player>(&mut app);
        let point = single::<SwitchPoint>(&mut app);
        let visited = app.world().resource::<TodayRoads>().0.clone();
        open_destinations(app.world_mut(), point, player);
        assert_removed(&app, &old_scene);
        assert_absent::<FallingObject>(&mut app);
        assert_absent::<ConstructionDetour>(&mut app);
        assert_eq!(app.world().resource::<RoadEffects>().0, effects);
        assert_eq!(app.world().resource::<TodayRoads>().0, visited);

        activate(&mut app, destination);
        let (root, _) = assert_road(&mut app, destination);
        let falling_object = single::<FallingObject>(&mut app);
        assert_ne!(falling_object, initial_falling_object);
        assert_eq!(
            app.world().get::<ChildOf>(falling_object).unwrap().parent(),
            root
        );
        if destination == RoadId::MarketRoad {
            let detour = single::<ConstructionDetour>(&mut app);
            assert_eq!(app.world().get::<ChildOf>(detour).unwrap().parent(), root);
        } else {
            assert_absent::<ConstructionDetour>(&mut app);
        }
        assert_eq!(app.world().resource::<RoadEffects>().0, effects);
        assert_today_roads(&app, &[RoadId::ExampleRoad, RoadId::MarketRoad]);
    }
}

#[test]
fn playing_loads_the_configured_initial_road_without_requiring_outgoing_connections() {
    let mut app = level_app();
    app.insert_state(GameState::Loading)
        .add_plugins(LevelPlugin);
    finish_app(&mut app);
    assert_absent::<SceneRoot>(&mut app);

    assert_eq!(app.world().resource::<InitialRoad>().0, RoadId::ExampleRoad);
    app.world_mut().resource_mut::<InitialRoad>().0 = RoadId::MarketRoad;
    app.world_mut()
        .resource_mut::<RoadGraph>()
        .roads
        .remove(&RoadId::MarketRoad);
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    assert_road(&mut app, RoadId::MarketRoad);
    assert_today_roads(&app, &[RoadId::MarketRoad]);
    assert_commute(&app, CommuteState::Running);
}
