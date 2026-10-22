use super::{
    COURT, EXAMPLE_ROAD, InteractRequested, LevelPlugin, NEXT_ROAD, ROAD_SPAWN, RoadId,
    SwitchPoint,
    navigation::{TravelUi, select_destination},
    ui::{TravelButtonAction, TravelUiRoot},
};
use crate::{
    player::{Player, PlayerPlugin},
    test_support::{advance, finish_app, headless_app},
};
use avian2d::prelude::*;
use bevy::{
    prelude::*,
    ui_widgets::{Activate, Button},
};
use leafwing_input_manager::prelude::*;

#[derive(Resource, Default)]
struct SwitchRequests(Vec<Entity>);

fn game(gravity: Vec2) -> (App, Entity, Entity) {
    let mut app = headless_app(gravity);
    app.add_plugins((PlayerPlugin, LevelPlugin))
        .init_resource::<SwitchRequests>()
        .add_observer(
            |request: On<InteractRequested>,
             points: Query<(), With<SwitchPoint>>,
             mut requests: ResMut<SwitchRequests>| {
                if points.contains(request.entity) {
                    requests.0.push(request.entity);
                }
            },
        );
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
            let children = children.to_vec();
            pending.extend(children.iter().copied());
            result.extend(children);
        }
    }
    result
}

fn position(app: &App, player: Entity) -> Vec2 {
    app.world().get::<Position>(player).unwrap().0
}

fn velocity(app: &App, player: Entity) -> Vec2 {
    app.world().get::<LinearVelocity>(player).unwrap().0
}

fn set_position(app: &mut App, player: Entity, position: Vec2) {
    app.world_mut().get_mut::<Position>(player).unwrap().0 = position;
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = position.extend(0.0);
    app.world_mut().get_mut::<LinearVelocity>(player).unwrap().0 = Vec2::ZERO;
}

fn overlap_exit(app: &mut App, player: Entity, offset: Vec2) -> Entity {
    let point = single::<SwitchPoint>(app);
    let center = app
        .world()
        .get::<GlobalTransform>(point)
        .unwrap()
        .translation()
        .truncate();
    set_position(app, player, center + offset);
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
    // The menu is rebuilt from the state changed by the BSN observer.
    app.update();
}

fn open_exit(app: &mut App, player: Entity) -> Entity {
    let point = overlap_exit(app, player, Vec2::ZERO);
    press_e(app);
    assert!(matches!(
        app.world().resource::<TravelUi>(),
        TravelUi::Choosing { .. }
    ));
    point
}

fn destinations(app: &mut App) -> Vec<RoadId> {
    let world = app.world_mut();
    let mut result: Vec<_> = world
        .query::<&TravelButtonAction>()
        .iter(world)
        .filter_map(|action| match action {
            TravelButtonAction::Destination(road) => Some(*road),
            TravelButtonAction::Close => None,
        })
        .collect();
    result.sort_unstable_by_key(|road| road.0);
    result
}

fn destination_button(app: &mut App, destination: RoadId) -> Entity {
    let world = app.world_mut();
    world
        .query::<(Entity, &TravelButtonAction)>()
        .iter(world)
        .find_map(|(entity, action)| {
            matches!(action, TravelButtonAction::Destination(road) if *road == destination)
                .then_some(entity)
        })
        .expect("the offered destination must have a button")
}

fn close_button(app: &mut App) -> Entity {
    let world = app.world_mut();
    world
        .query::<(Entity, &TravelButtonAction)>()
        .iter(world)
        .find_map(|(entity, action)| matches!(action, TravelButtonAction::Close).then_some(entity))
        .expect("an open menu must offer cancellation")
}

fn activate(app: &mut App, button: Entity) {
    assert!(app.world().get::<Button>(button).is_some());
    assert!(app.world().get::<Node>(button).is_some());
    app.world_mut().trigger(Activate { entity: button });
    app.world_mut().flush();
}

fn assert_closed(app: &mut App) {
    assert!(matches!(
        app.world().resource::<TravelUi>(),
        TravelUi::Closed
    ));
    assert!(!app.world().resource::<Time<Physics>>().is_paused());
    assert_eq!(count::<TravelUiRoot>(app), 0);
    assert_eq!(count::<TravelButtonAction>(app), 0);
}

#[test]
fn e_and_native_buttons_complete_the_forward_road_and_court_flow() {
    let (mut app, player, camera) = game(Vec2::ZERO);
    let unrelated = app.world_mut().spawn_empty().id();
    let first_road = single::<RoadId>(&mut app);
    assert_eq!(app.world().get::<RoadId>(first_road), Some(&EXAMPLE_ROAD));
    let old_children = descendants(app.world(), first_road);
    assert!(!old_children.is_empty());
    assert_eq!(position(&app, player), ROAD_SPAWN);

    let first_exit = open_exit(&mut app, player);
    assert_eq!(
        app.world()
            .get::<SwitchPoint>(first_exit)
            .unwrap()
            .destinations,
        &[NEXT_ROAD, COURT]
    );
    assert_eq!(destinations(&mut app), [COURT, NEXT_ROAD]);
    let menu = single::<TravelUiRoot>(&mut app);
    let button = destination_button(&mut app, NEXT_ROAD);
    assert!(descendants(app.world(), menu).contains(&button));
    assert!(app.world().resource::<Time<Physics>>().is_paused());
    // A transition must discard momentum even if another system wrote it while paused.
    app.world_mut().get_mut::<LinearVelocity>(player).unwrap().0 = Vec2::new(220.0, -70.0);
    activate(&mut app, button);

    let next_road = single::<RoadId>(&mut app);
    assert_ne!(next_road, first_road);
    assert_eq!(app.world().get::<RoadId>(next_road), Some(&NEXT_ROAD));
    assert!(app.world().get_entity(first_road).is_err());
    assert!(
        old_children
            .iter()
            .all(|entity| app.world().get_entity(*entity).is_err())
    );
    assert_eq!(count::<RoadId>(&mut app), 1);
    assert_eq!(single::<Player>(&mut app), player);
    assert_eq!(single::<Camera2d>(&mut app), camera);
    assert!(app.world().get_entity(unrelated).is_ok());
    assert_eq!(position(&app, player), ROAD_SPAWN);
    assert_eq!(
        app.world()
            .get::<Transform>(player)
            .unwrap()
            .translation
            .truncate(),
        ROAD_SPAWN
    );
    assert_eq!(velocity(&app, player), Vec2::ZERO);
    assert!(app.world().get::<ShapeHits>(player).unwrap().is_empty());
    app.update();
    assert_closed(&mut app);

    let next_exit = open_exit(&mut app, player);
    assert_eq!(
        app.world()
            .get::<SwitchPoint>(next_exit)
            .unwrap()
            .destinations,
        &[COURT]
    );
    assert_eq!(destinations(&mut app), [COURT]);
    let next_children = descendants(app.world(), next_road);
    let court_position = position(&app, player);
    let button = destination_button(&mut app, COURT);
    activate(&mut app, button);
    app.update();

    assert!(matches!(
        app.world().resource::<TravelUi>(),
        TravelUi::Court
    ));
    assert!(app.world().resource::<Time<Physics>>().is_paused());
    assert_eq!(single::<RoadId>(&mut app), next_road);
    assert_eq!(descendants(app.world(), next_road), next_children);
    assert_eq!(single::<Player>(&mut app), player);
    assert_eq!(single::<Camera2d>(&mut app), camera);
    assert_eq!(position(&app, player), court_position);
    assert!(destinations(&mut app).is_empty());
    assert_eq!(count::<TravelUiRoot>(&mut app), 1);
    let close = close_button(&mut app);
    activate(&mut app, close);
    app.update();
    assert_closed(&mut app);
}

#[test]
fn an_open_menu_freezes_motion_gravity_and_repeated_e_until_escape() {
    let (mut app, player, _) = game(Vec2::NEG_Y * 1000.0);
    let road = single::<RoadId>(&mut app);
    overlap_exit(&mut app, player, Vec2::Y * 25.0);
    assert!(
        velocity(&app, player).y < 0.0,
        "the player is falling before opening"
    );
    press_e(&mut app);
    let menu = single::<TravelUiRoot>(&mut app);
    let frozen_position = position(&app, player);
    assert_eq!(velocity(&app, player), Vec2::ZERO);
    assert_eq!(app.world().resource::<SwitchRequests>().0.len(), 1);

    KeyCode::KeyD.press(app.world_mut());
    KeyCode::Space.press(app.world_mut());
    advance(&mut app, 12);
    press_e(&mut app);
    assert_eq!(position(&app, player), frozen_position);
    assert_eq!(velocity(&app, player), Vec2::ZERO);
    assert_eq!(app.world().resource::<SwitchRequests>().0.len(), 1);
    assert_eq!(single::<TravelUiRoot>(&mut app), menu);
    assert_eq!(single::<RoadId>(&mut app), road);

    KeyCode::KeyD.release(app.world_mut());
    KeyCode::Space.release(app.world_mut());
    KeyCode::Escape.press(app.world_mut());
    app.update();
    app.update();
    assert_closed(&mut app);
    assert_eq!(single::<RoadId>(&mut app), road);
    let before_move = position(&app, player);
    KeyCode::KeyD.press(app.world_mut());
    app.update();
    assert!(
        position(&app, player).x > before_move.x,
        "cancellation restores movement"
    );
    assert!(
        velocity(&app, player).y < 0.0,
        "cancellation restores gravity"
    );
}

#[test]
fn cancelling_either_menu_preserves_the_current_road() {
    let (mut app, player, _) = game(Vec2::ZERO);
    let road = single::<RoadId>(&mut app);
    let children = descendants(app.world(), road);
    open_exit(&mut app, player);
    let before_cancel = position(&app, player);
    let close = close_button(&mut app);
    activate(&mut app, close);
    app.update();
    assert_closed(&mut app);
    assert_eq!(single::<RoadId>(&mut app), road);
    assert_eq!(descendants(app.world(), road), children);
    assert_eq!(position(&app, player), before_cancel);

    open_exit(&mut app, player);
    let court = destination_button(&mut app, COURT);
    activate(&mut app, court);
    app.update();
    assert!(matches!(
        app.world().resource::<TravelUi>(),
        TravelUi::Court
    ));
    KeyCode::Escape.press(app.world_mut());
    advance(&mut app, 2);
    assert_closed(&mut app);
    assert_eq!(single::<RoadId>(&mut app), road);
    assert_eq!(descendants(app.world(), road), children);
}

#[test]
fn closing_a_menu_with_a_fresh_e_press_requires_another_press_to_reopen() {
    use bevy::ecs::schedule::common_conditions::run_once;

    for escape in [true, false] {
        let (mut app, player, _) = game(Vec2::ZERO);
        open_exit(&mut app, player);
        KeyCode::KeyE.release(app.world_mut());
        app.update();
        KeyCode::KeyE.press(app.world_mut());
        if escape {
            KeyCode::Escape.press(app.world_mut());
        } else {
            let close = close_button(&mut app);
            app.add_systems(
                Update,
                (move |mut commands: Commands| {
                    commands.trigger(Activate { entity: close });
                })
                .run_if(run_once),
            );
        }
        advance(&mut app, 3);
        assert_closed(&mut app);
        assert_eq!(app.world().resource::<SwitchRequests>().0.len(), 1);

        KeyCode::Escape.release(app.world_mut());
        press_e(&mut app);
        assert!(app.world().resource::<TravelUi>().is_open());
        assert_eq!(app.world().resource::<SwitchRequests>().0.len(), 2);
    }
}

#[test]
fn unavailable_or_unknown_destinations_never_unload_the_current_road() {
    const UNKNOWN: RoadId = RoadId("unknown_road");
    const UNKNOWN_DESTINATIONS: &[RoadId] = &[UNKNOWN];
    let (mut app, player, _) = game(Vec2::ZERO);
    let road = single::<RoadId>(&mut app);
    select_destination(app.world_mut(), NEXT_ROAD);
    select_destination(app.world_mut(), COURT);
    assert!(matches!(
        app.world().resource::<TravelUi>(),
        TravelUi::Closed
    ));
    assert_eq!(single::<RoadId>(&mut app), road);

    let point = open_exit(&mut app, player);
    for destination in [EXAMPLE_ROAD, UNKNOWN] {
        select_destination(app.world_mut(), destination);
        assert!(matches!(
            app.world().resource::<TravelUi>(),
            TravelUi::Choosing { .. }
        ));
        assert_eq!(single::<RoadId>(&mut app), road);
    }
    let close = close_button(&mut app);
    activate(&mut app, close);
    app.update();
    app.world_mut()
        .get_mut::<SwitchPoint>(point)
        .unwrap()
        .destinations = UNKNOWN_DESTINATIONS;
    press_e(&mut app);
    assert_eq!(
        destinations(&mut app),
        [UNKNOWN],
        "the observer reads each point's own configuration"
    );
    let unknown = destination_button(&mut app, UNKNOWN);
    activate(&mut app, unknown);
    app.update();
    assert!(matches!(
        app.world().resource::<TravelUi>(),
        TravelUi::Choosing { .. }
    ));
    assert_eq!(single::<RoadId>(&mut app), road);
    assert_eq!(count::<RoadId>(&mut app), 1);
}
