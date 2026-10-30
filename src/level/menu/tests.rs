use super::*;
use crate::{
    config::GameSettings,
    game::{CommuteState, CommuteTime, GamePlugin, HasHelmet, Money, Stamina},
    level::{InitialRoad, LevelPlugin, RoadId, ui::TravelUiRoot},
    player::Player,
    test_support::{advance, finish_app, insert_config, scene_app},
};
use bevy::{state::app::StatesPlugin, time::TimeUpdateStrategy};
use std::time::Duration;

fn single<C: Component>(app: &mut App) -> Entity {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<C>>()
        .single(world)
        .unwrap()
}

fn assert_no_gameplay(app: &mut App) {
    let world = app.world_mut();
    assert!(!world.contains_resource::<CommuteTime>());
    assert!(!world.contains_resource::<Money>());
    assert!(!world.contains_resource::<Stamina>());
    assert!(!world.contains_resource::<HasHelmet>());
    assert_eq!(world.query::<&Player>().iter(world).count(), 0);
    assert_eq!(world.query::<&RoadId>().iter(world).count(), 0);
}

#[test]
fn menu_waits_for_start_then_loads_the_initial_road_and_cleans_up_its_entities() {
    let mut app = scene_app();
    app.add_plugins(StatesPlugin);
    insert_config(&mut app);
    app.insert_state(GameState::Loading)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            125,
        )))
        .add_plugins((GamePlugin, LevelPlugin));
    let camera = app.world_mut().spawn(Camera2d).id();
    finish_app(&mut app);
    assert_no_gameplay(&mut app);
    {
        let world = app.world_mut();
        assert_eq!(world.query::<&SceneRoot>().iter(world).count(), 0);
    }

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

    let mut menu_entities = vec![root];
    let mut index = 0;
    while index < menu_entities.len() {
        if let Some(children) = app.world().get::<Children>(menu_entities[index]) {
            menu_entities.extend(children.iter());
        }
        index += 1;
    }
    // Multiple activations before the transition still start only one game.
    app.world_mut().trigger(Activate { entity: button });
    app.world_mut().trigger(Activate { entity: button });
    app.update();

    assert_eq!(
        *app.world().resource::<State<GameState>>().get(),
        GameState::Playing
    );
    for entity in menu_entities {
        assert!(app.world().get_entity(entity).is_err());
    }
    assert!(app.world().get_entity(camera).is_ok());
    let road = single::<RoadId>(&mut app);
    assert_eq!(app.world().get::<RoadId>(road), Some(&RoadId::ExampleRoad));
    assert_eq!(app.world().resource::<InitialRoad>().0, RoadId::ExampleRoad);
    let player = single::<Player>(&mut app);
    assert_eq!(app.world().get::<ChildOf>(player).unwrap().parent(), road);
    assert_eq!(single::<SceneRoot>(&mut app), road);
    {
        let world = app.world_mut();
        assert_eq!(world.query::<&TravelUiRoot>().iter(world).count(), 0);
    }
    assert_eq!(
        app.world().resource::<Money>().0,
        app.world().resource::<GameSettings>().initial_money
    );
    assert_eq!(app.world().resource::<Stamina>().0, Stamina::MAX);
    assert!(!app.world().resource::<HasHelmet>().0);
    advance(&mut app, 2);
    assert_eq!(
        *app.world().resource::<State<CommuteState>>().get(),
        CommuteState::Running
    );
    assert!(app.world().resource::<CommuteTime>().elapsed > 0.0);
}
