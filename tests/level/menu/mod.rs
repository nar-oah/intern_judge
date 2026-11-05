use super::*;
use crate::{
    config::{GameSettings, RoadGraph},
    game::{CommuteState, CommuteTime, GamePlugin, HasHelmet, Money, Stamina},
    level::{
        InitialRoad, LevelPlugin, RoadId, RoadUnlocks,
        ui::{TravelButtonAction, TravelUiRoot},
    },
    player::Player,
    test_support::{
        advance, assert_absent, assert_removed, finish_app, level_app, scene_entities, single,
    },
};
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

fn assert_no_gameplay(app: &mut App) {
    let world = app.world_mut();
    assert!(!world.contains_resource::<CommuteTime>());
    assert!(!world.contains_resource::<Money>());
    assert!(!world.contains_resource::<Stamina>());
    assert!(!world.contains_resource::<HasHelmet>());
    assert_absent::<Player>(app);
    assert_absent::<RoadId>(app);
}

#[test]
fn menu_waits_for_start_then_opens_the_weekend_unlock_map_and_cleans_up_its_entities() {
    let mut app = level_app();
    app.world_mut()
        .resource_mut::<RoadGraph>()
        .initial_unlocks
        .insert(RoadId::MarketRoad, false);
    app.insert_state(GameState::Loading)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            125,
        )))
        .add_plugins((GamePlugin, LevelPlugin));
    let camera = app.world_mut().spawn(Camera2d).id();
    finish_app(&mut app);
    assert_no_gameplay(&mut app);
    assert_absent::<SceneRoot>(&mut app);

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::MainMenu);
    app.update();
    let root = single::<MainMenuRoot>(&mut app);
    assert!(app.world().get::<SceneRoot>(root).is_some());
    let button = single::<Button>(&mut app);
    {
        let world = app.world_mut();
        let labels = world
            .query::<&Text>()
            .iter(world)
            .map(|text| text.0.as_str())
            .collect::<Vec<_>>();
        assert!(labels.contains(&"Intern Judge"));
        assert!(labels.contains(&"Start Game"));
    }
    advance(&mut app, 8);
    assert_eq!(
        *app.world().resource::<State<GameState>>().get(),
        GameState::MainMenu
    );
    assert_no_gameplay(&mut app);

    let menu_entities = scene_entities(app.world(), root);
    // Multiple activations before the transition still start only one game.
    app.world_mut().trigger(Activate { entity: button });
    app.world_mut().trigger(Activate { entity: button });
    app.update();

    assert_eq!(
        *app.world().resource::<State<GameState>>().get(),
        GameState::Playing
    );
    assert_removed(&app, &menu_entities);
    assert!(app.world().get_entity(camera).is_ok());
    assert_eq!(app.world().resource::<InitialRoad>().0, RoadId::ExampleRoad);
    let map = single::<TravelUiRoot>(&mut app);
    assert_eq!(single::<SceneRoot>(&mut app), map);
    assert_eq!(
        app.world().get::<TravelUiRoot>(map).unwrap().0,
        [RoadId::MarketRoad]
    );
    assert_absent::<Player>(&mut app);
    assert_absent::<RoadId>(&mut app);
    assert_eq!(
        app.world().resource::<Money>().0,
        app.world().resource::<GameSettings>().initial_money
    );
    assert_eq!(app.world().resource::<Stamina>().0, Stamina::MAX);
    assert!(!app.world().resource::<HasHelmet>().0);
    advance(&mut app, 2);
    assert_eq!(
        *app.world().resource::<State<CommuteState>>().get(),
        CommuteState::Paused
    );
    let paused_at = app.world().resource::<CommuteTime>().elapsed;
    let destination = single::<TravelButtonAction>(&mut app);
    app.world_mut().trigger(Activate {
        entity: destination,
    });
    advance(&mut app, 2);
    let road = single::<RoadId>(&mut app);
    assert_eq!(app.world().get::<RoadId>(road), Some(&RoadId::MarketRoad));
    let player = single::<Player>(&mut app);
    assert_eq!(app.world().get::<ChildOf>(player).unwrap().parent(), road);
    assert!(app.world().resource::<RoadUnlocks>().0[&RoadId::MarketRoad]);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, paused_at);
}
