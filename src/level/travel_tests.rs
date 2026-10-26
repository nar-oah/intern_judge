use super::{
    InteractRequested, LevelPlugin, RoadId, SceneRoot, SwitchPoint,
    navigation::{open_destinations, select_destination},
    scene::{COURT, EXAMPLE_ROAD, MARKET_ROAD},
    ui::{TravelButtonAction, TravelUiRoot},
};
use crate::{
    game::{CommuteState, CommuteTime, GamePlugin},
    player::{Player, PlayerPlugin},
    test_support::{advance, finish_app, headless_app},
};
use avian2d::prelude::*;
use bevy::{
    prelude::*,
    state::app::StatesPlugin,
    ui_widgets::{Activate, Button},
};
use leafwing_input_manager::prelude::*;

const SPAWN: Vec2 = Vec2::new(-460.0, -130.0);
const UNKNOWN: RoadId = RoadId("unknown_road");

fn game() -> (App, Entity, Entity) {
    let mut app = headless_app(Vec2::ZERO);
    app.add_plugins((StatesPlugin, GamePlugin, PlayerPlugin, LevelPlugin));
    let camera = app.world_mut().spawn(Camera2d).id();
    finish_app(&mut app);
    advance(&mut app, 2);
    let player = single::<Player>(&mut app);
    (app, player, camera)
}

fn single<C: Component>(app: &mut App) -> Entity {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<C>>()
        .single(world)
        .unwrap()
}

fn count<C: Component>(app: &mut App) -> usize {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<C>>()
        .iter(world)
        .count()
}

fn descendants(world: &World, root: Entity) -> Vec<Entity> {
    let mut result = Vec::new();
    let mut pending = vec![root];
    while let Some(parent) = pending.pop() {
        if let Some(children) = world.get::<Children>(parent) {
            pending.extend(children.iter());
            result.extend(children.iter());
        }
    }
    result
}

fn assert_removed(app: &App, root: Entity, children: &[Entity]) {
    assert!(app.world().get_entity(root).is_err());
    assert!(
        children
            .iter()
            .all(|&child| app.world().get_entity(child).is_err())
    );
}

fn position(app: &App, player: Entity) -> Vec2 {
    app.world().get::<Position>(player).unwrap().0
}

fn overlap_exit(app: &mut App, player: Entity) -> Entity {
    let point = single::<SwitchPoint>(app);
    let center = app
        .world()
        .get::<GlobalTransform>(point)
        .unwrap()
        .translation();
    app.world_mut().get_mut::<Position>(player).unwrap().0 = center.truncate();
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = center;
    app.world_mut().get_mut::<LinearVelocity>(player).unwrap().0 = Vec2::ZERO;
    advance(app, 3);
    assert!(
        app.world()
            .get::<CollidingEntities>(point)
            .unwrap()
            .contains(&player)
    );
    point
}

fn press_e(app: &mut App) {
    KeyCode::KeyE.release(app.world_mut());
    app.update();
    KeyCode::KeyE.press(app.world_mut());
    app.update();
}

fn destination_button(app: &mut App, destination: RoadId) -> Entity {
    let world = app.world_mut();
    world
        .query::<(Entity, &TravelButtonAction)>()
        .iter(world)
        .find_map(|(entity, action)| (action.0 == destination).then_some(entity))
        .expect("the offered destination must have a button")
}

fn activate(app: &mut App, button: Entity) {
    assert!(app.world().get::<Button>(button).is_some());
    assert!(app.world().get::<Node>(button).is_some());
    app.world_mut().trigger(Activate { entity: button });
    app.world_mut().flush();
}

fn assert_road(app: &mut App, destination: RoadId) -> (Entity, Entity) {
    let road = single::<RoadId>(app);
    let player = single::<Player>(app);
    assert_eq!(single::<SceneRoot>(app), road);
    assert_eq!(app.world().get::<RoadId>(road), Some(&destination));
    assert_eq!(app.world().get::<ChildOf>(player).unwrap().parent(), road);
    assert_eq!(count::<TravelUiRoot>(app), 0);
    assert_eq!(count::<TravelButtonAction>(app), 0);
    assert_eq!(position(app, player), SPAWN);
    assert_eq!(
        app.world()
            .get::<Transform>(player)
            .unwrap()
            .translation
            .truncate(),
        SPAWN
    );
    assert_eq!(
        app.world().get::<LinearVelocity>(player).unwrap().0,
        Vec2::ZERO
    );
    assert_eq!(app.world().get::<AngularVelocity>(player).unwrap().0, 0.0);
    assert!(app.world().get::<ShapeHits>(player).unwrap().is_empty());
    (road, player)
}

fn assert_ui(app: &mut App, destinations: &[RoadId]) -> Entity {
    let root = single::<TravelUiRoot>(app);
    assert_eq!(single::<SceneRoot>(app), root);
    assert_eq!(
        app.world().get::<TravelUiRoot>(root).unwrap().0,
        destinations
    );
    assert_eq!(count::<RoadId>(app), 0);
    assert_eq!(count::<Player>(app), 0);
    assert_eq!(count::<super::Platform>(app), 0);
    assert_eq!(count::<super::BuildingType>(app), 0);
    assert_eq!(count::<SwitchPoint>(app), 0);
    assert_eq!(count::<Button>(app), destinations.len());
    assert_eq!(count::<TravelButtonAction>(app), destinations.len());
    let node = app.world().get::<Node>(root).unwrap();
    assert_eq!(node.width, percent(100));
    assert_eq!(node.height, percent(100));
    assert_eq!(node.position_type, PositionType::Absolute);
    assert_eq!(
        app.world().get::<BackgroundColor>(root).unwrap().0.alpha(),
        1.0
    );
    let children = descendants(app.world(), root);
    for &destination in destinations {
        let button = destination_button(app, destination);
        assert!(children.contains(&button));
        assert!(app.world().get::<Button>(button).is_some());
    }
    root
}

#[test]
fn commute_pauses_for_destinations_resumes_on_roads_and_stays_paused_in_court() {
    let (mut app, player, _) = game();
    assert_eq!(
        app.world().resource::<State<CommuteState>>().get(),
        &CommuteState::Running
    );
    let before = app.world().resource::<CommuteTime>().elapsed;
    advance(&mut app, 4);
    assert!(app.world().resource::<CommuteTime>().elapsed > before);

    for destination in [MARKET_ROAD, COURT] {
        let player = if destination == MARKET_ROAD {
            player
        } else {
            single::<Player>(&mut app)
        };
        overlap_exit(&mut app, player);
        press_e(&mut app);
        // Navigation requests a state change; the next frame applies it.
        app.update();
        assert_eq!(
            app.world().resource::<State<CommuteState>>().get(),
            &CommuteState::Paused
        );
        let paused_at = app.world().resource::<CommuteTime>().elapsed;
        advance(&mut app, 4);
        assert_eq!(app.world().resource::<CommuteTime>().elapsed, paused_at);

        let button = destination_button(&mut app, destination);
        activate(&mut app, button);
        app.update();
        advance(&mut app, 4);
        if destination == COURT {
            assert_eq!(
                app.world().resource::<State<CommuteState>>().get(),
                &CommuteState::Paused
            );
            assert_eq!(app.world().resource::<CommuteTime>().elapsed, paused_at);
        } else {
            assert_eq!(
                app.world().resource::<State<CommuteState>>().get(),
                &CommuteState::Running
            );
            assert!(app.world().resource::<CommuteTime>().elapsed > paused_at);
        }
        let road = single::<RoadId>(&mut app);
        assert_eq!(app.world().get::<RoadId>(road), Some(&destination));
    }
}

#[test]
fn e_and_native_buttons_replace_complete_scenes_and_recreate_players() {
    let (mut app, mut player, camera) = game();
    let unrelated = app.world_mut().spawn_empty().id();
    let (mut road, _) = assert_road(&mut app, EXAMPLE_ROAD);

    for (destination, choices) in [
        (MARKET_ROAD, &[MARKET_ROAD, COURT][..]),
        (COURT, &[COURT][..]),
    ] {
        let children = descendants(app.world(), road);
        assert!(children.contains(&player));
        let point = overlap_exit(&mut app, player);
        assert_eq!(
            app.world().get::<SwitchPoint>(point).unwrap().destinations,
            choices
        );
        app.world_mut().get_mut::<LinearVelocity>(player).unwrap().0 = Vec2::new(220.0, -70.0);
        app.world_mut()
            .get_mut::<AngularVelocity>(player)
            .unwrap()
            .0 = 3.0;
        press_e(&mut app);

        let menu = assert_ui(&mut app, choices);
        assert_removed(&app, road, &children);
        assert!(app.world().get_entity(player).is_err());
        assert_eq!(single::<Camera2d>(&mut app), camera);
        assert!(app.world().get_entity(unrelated).is_ok());
        assert!(!app.world().resource::<Time<Physics>>().is_paused());

        // With no Player, movement and interaction naturally do nothing in the UI.
        KeyCode::KeyD.press(app.world_mut());
        KeyCode::Space.press(app.world_mut());
        advance(&mut app, 4);
        press_e(&mut app);
        assert_eq!(assert_ui(&mut app, choices), menu);
        KeyCode::KeyD.release(app.world_mut());
        KeyCode::Space.release(app.world_mut());
        app.update();

        let menu_children = descendants(app.world(), menu);
        let button = destination_button(&mut app, destination);
        activate(&mut app, button);
        assert_removed(&app, menu, &menu_children);
        let (next_road, next_player) = assert_road(&mut app, destination);
        assert_ne!(next_road, road);
        assert_ne!(next_player, player);
        assert_eq!(single::<Camera2d>(&mut app), camera);
        assert!(app.world().get_entity(unrelated).is_ok());
        road = next_road;
        player = next_player;
        advance(&mut app, 2);
    }

    assert_eq!(count::<super::Platform>(&mut app), 1);
    assert_eq!(count::<super::BuildingType>(&mut app), 0);
    assert_eq!(count::<SwitchPoint>(&mut app), 0);
    assert_eq!(descendants(app.world(), road).len(), 2);

    app.world_mut().resource_mut::<Gravity>().0 = Vec2::NEG_Y * 1000.0;
    advance(&mut app, 32);
    assert!(!app.world().get::<ShapeHits>(player).unwrap().is_empty());
    let before_move = position(&app, player);
    KeyCode::KeyD.press(app.world_mut());
    app.update();
    assert!(position(&app, player).x > before_move.x);
    KeyCode::KeyD.release(app.world_mut());
    KeyCode::Space.press(app.world_mut());
    app.update();
    assert!(app.world().get::<LinearVelocity>(player).unwrap().y > 350.0);
    assert!(position(&app, player).y > before_move.y);
}

#[test]
fn empty_or_unknown_destinations_keep_the_road_and_player() {
    for destinations in [&[][..], &[UNKNOWN][..]] {
        let (mut app, player, _) = game();
        let road = single::<RoadId>(&mut app);
        let children = descendants(app.world(), road);
        let point = overlap_exit(&mut app, player);
        app.world_mut()
            .get_mut::<SwitchPoint>(point)
            .unwrap()
            .destinations = destinations;
        press_e(&mut app);
        app.update();
        assert_eq!(single::<SceneRoot>(&mut app), road);
        assert_eq!(single::<Player>(&mut app), player);
        assert_eq!(count::<TravelUiRoot>(&mut app), 0);
        assert_eq!(descendants(app.world(), road), children);
        assert_eq!(
            app.world().resource::<State<CommuteState>>().get(),
            &CommuteState::Running
        );
    }
}

#[test]
fn only_valid_unique_destinations_are_offered_and_accepted() {
    let (mut app, player, _) = game();
    let road = single::<SceneRoot>(&mut app);
    let point = overlap_exit(&mut app, player);
    let orphan = app.world_mut().spawn(TravelButtonAction(MARKET_ROAD)).id();
    select_destination(app.world_mut(), orphan);
    open_destinations(app.world_mut(), point, Entity::PLACEHOLDER);
    open_destinations(app.world_mut(), road, player);
    app.update();
    assert_eq!(single::<SceneRoot>(&mut app), road);
    assert_eq!(
        app.world().resource::<State<CommuteState>>().get(),
        &CommuteState::Running
    );

    app.world_mut()
        .get_mut::<SwitchPoint>(point)
        .unwrap()
        .destinations = &[UNKNOWN, MARKET_ROAD, MARKET_ROAD, COURT];
    press_e(&mut app);
    // The unrelated action is not a UI button and cannot select a destination.
    app.world_mut().despawn(orphan);
    let menu = assert_ui(&mut app, &[MARKET_ROAD, COURT]);
    app.update();
    let button = destination_button(&mut app, MARKET_ROAD);
    for invalid in [EXAMPLE_ROAD, UNKNOWN] {
        app.world_mut()
            .get_mut::<TravelButtonAction>(button)
            .unwrap()
            .0 = invalid;
        activate(&mut app, button);
        app.update();
        assert_eq!(single::<SceneRoot>(&mut app), menu);
        assert_eq!(count::<RoadId>(&mut app), 0);
        assert_eq!(
            app.world().resource::<State<CommuteState>>().get(),
            &CommuteState::Paused
        );
    }
    // Even a corrupted choice list must not unload the UI for an unknown road.
    app.world_mut()
        .get_mut::<TravelUiRoot>(menu)
        .unwrap()
        .0
        .push(UNKNOWN);
    activate(&mut app, button);
    app.update();
    assert_eq!(single::<SceneRoot>(&mut app), menu);
    assert_eq!(
        app.world().resource::<State<CommuteState>>().get(),
        &CommuteState::Paused
    );
    app.world_mut()
        .get_mut::<TravelButtonAction>(button)
        .unwrap()
        .0 = MARKET_ROAD;
    activate(&mut app, button);
    assert_road(&mut app, MARKET_ROAD);
}

#[test]
fn duplicate_and_stale_observer_events_do_not_replace_the_new_scene() {
    let (mut app, player, _) = game();
    let road = single::<SceneRoot>(&mut app);
    let children = descendants(app.world(), road);
    let point = overlap_exit(&mut app, player);
    {
        let mut commands = app.world_mut().commands();
        for _ in 0..3 {
            commands.trigger(InteractRequested {
                entity: point,
                player,
            });
        }
    }
    app.world_mut().flush();
    let menu = assert_ui(&mut app, &[MARKET_ROAD, COURT]);
    assert_removed(&app, road, &children);
    let market = destination_button(&mut app, MARKET_ROAD);
    let court = destination_button(&mut app, COURT);
    {
        let mut commands = app.world_mut().commands();
        for button in [market, court, market] {
            commands.trigger(Activate { entity: button });
        }
    }
    app.world_mut().flush();
    let (next_road, next_player) = assert_road(&mut app, MARKET_ROAD);
    assert!(app.world().get_entity(menu).is_err());
    select_destination(app.world_mut(), court);
    open_destinations(app.world_mut(), point, player);
    app.update();
    assert_eq!(single::<SceneRoot>(&mut app), next_road);
    assert_eq!(single::<Player>(&mut app), next_player);
    assert_eq!(
        app.world().resource::<State<CommuteState>>().get(),
        &CommuteState::Running
    );

    overlap_exit(&mut app, next_player);
    press_e(&mut app);
    let next_menu = assert_ui(&mut app, &[COURT]);
    select_destination(app.world_mut(), court);
    app.update();
    assert_eq!(single::<SceneRoot>(&mut app), next_menu);
    assert_eq!(
        app.world().resource::<State<CommuteState>>().get(),
        &CommuteState::Paused
    );
    let new_court = destination_button(&mut app, COURT);
    activate(&mut app, new_court);
    assert_road(&mut app, COURT);
}
