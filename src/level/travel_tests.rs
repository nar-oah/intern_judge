use super::{
    BuildingType, InteractRequested, LevelPlugin, RoadId, SceneRoot, SwitchPoint,
    map::MapLayout,
    navigation::{destinations, open_destinations, select_destination},
    scene,
    ui::{TravelButtonAction, TravelUiRoot},
};
use crate::{
    config::{BuildingValues, ConvenienceKind, GameSettings, GameState, RoadGraph},
    game::{CommuteState, CommuteTime, GamePlugin, HasHelmet, Money, Stamina},
    player::{Player, PlayerPlugin},
    test_support::{advance, finish_app, headless_app, insert_config, scene_app},
};
use avian2d::prelude::*;
use bevy::{prelude::*, state::app::StatesPlugin, ui_widgets::Activate};
use leafwing_input_manager::prelude::*;

fn single<C: Component>(app: &mut App) -> Entity {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<C>>()
        .single(world)
        .unwrap()
}

fn assert_absent<C: Component>(app: &mut App) {
    let world = app.world_mut();
    assert!(
        world
            .query_filtered::<Entity, With<C>>()
            .iter(world)
            .next()
            .is_none()
    );
}

fn scene_entities(world: &World, root: Entity) -> Vec<Entity> {
    let mut entities = vec![root];
    let mut index = 0;
    while index < entities.len() {
        if let Some(children) = world.get::<Children>(entities[index]) {
            entities.extend(children.iter());
        }
        index += 1;
    }
    entities
}

fn assert_removed(app: &App, entities: &[Entity]) {
    for &entity in entities {
        assert!(
            app.world().get_entity(entity).is_err(),
            "stale scene entity {entity:?}"
        );
    }
}

fn destination_button(app: &mut App, destination: RoadId) -> Entity {
    let world = app.world_mut();
    world
        .query::<(Entity, &TravelButtonAction)>()
        .iter(world)
        .find_map(|(entity, action)| (action.0 == destination).then_some(entity))
        .expect("the destination must be offered")
}

fn activate(app: &mut App, destination: RoadId) {
    let button = destination_button(app, destination);
    app.world_mut().trigger(Activate { entity: button });
    app.world_mut().flush();
}

fn assert_map(app: &mut App, choices: &[RoadId]) -> Entity {
    let root = single::<TravelUiRoot>(app);
    assert_eq!(app.world().get::<TravelUiRoot>(root).unwrap().0, choices);
    assert_absent::<Player>(app);
    root
}

fn assert_road(app: &mut App, destination: RoadId) -> (Entity, Entity) {
    let root = single::<SceneRoot>(app);
    let player = single::<Player>(app);
    assert_eq!(app.world().get::<RoadId>(root), Some(&destination));
    assert_eq!(app.world().get::<ChildOf>(player).unwrap().parent(), root);
    assert_absent::<TravelUiRoot>(app);
    (root, player)
}

fn teleport(app: &mut App, entity: Entity, position: Vec3) {
    let local = app
        .world()
        .get::<ChildOf>(entity)
        .and_then(|parent| app.world().get::<GlobalTransform>(parent.parent()))
        .map_or(position, |parent| {
            parent.affine().inverse().transform_point3(position)
        });
    app.world_mut()
        .get_mut::<Transform>(entity)
        .unwrap()
        .translation = local;
    app.world_mut().get_mut::<Position>(entity).unwrap().0 = position.truncate();
    app.world_mut().get_mut::<LinearVelocity>(entity).unwrap().0 = Vec2::ZERO;
    advance(app, 3);
}

fn press_e(app: &mut App) {
    KeyCode::KeyE.release(app.world_mut());
    app.update();
    KeyCode::KeyE.press(app.world_mut());
    app.update();
}

fn assert_commute(app: &App, state: CommuteState) {
    assert_eq!(*app.world().resource::<State<CommuteState>>().get(), state);
}

fn level_app() -> App {
    let mut app = scene_app();
    app.add_plugins(StatesPlugin).init_state::<CommuteState>();
    insert_config(&mut app);
    app
}

// Navigation rejection needs real scene ancestry and observers, without advancing physics.
fn navigation_app() -> App {
    let mut app = level_app();
    let layout = MapLayout::from_graph(app.world().resource::<RoadGraph>());
    app.insert_resource(layout);
    finish_app(&mut app);
    app.world_mut()
        .spawn_scene(scene::road_scene(RoadId::ExampleRoad).unwrap())
        .unwrap();
    app
}

#[test]
fn playing_starts_with_destinations_from_the_current_start_connections() {
    let mut app = level_app();
    app.insert_state(GameState::Loading)
        .add_plugins(LevelPlugin);
    finish_app(&mut app);
    assert_absent::<SceneRoot>(&mut app);

    let mut graph = app.world_mut().resource_mut::<RoadGraph>();
    graph.roads.insert(
        RoadId::Start,
        vec![RoadId::Start, RoadId::MarketRoad, RoadId::MarketRoad],
    );
    graph.roads.remove(&RoadId::MarketRoad);
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    assert_map(&mut app, &[RoadId::MarketRoad]);
    assert_commute(&app, CommuteState::Paused);
    activate(&mut app, RoadId::MarketRoad);
    assert_road(&mut app, RoadId::MarketRoad);
}

#[test]
fn commute_journey_replaces_scenes_and_preserves_global_progress() {
    let mut app = headless_app(Vec2::ZERO);
    app.add_plugins((GamePlugin, PlayerPlugin, LevelPlugin));
    let camera = app.world_mut().spawn(Camera2d).id();
    let unrelated = app.world_mut().spawn_empty().id();
    finish_app(&mut app);

    let initial_map = assert_map(&mut app, &[RoadId::ExampleRoad]);
    let old_map = scene_entities(app.world(), initial_map);
    advance(&mut app, 4);
    assert_commute(&app, CommuteState::Paused);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.0);
    activate(&mut app, RoadId::ExampleRoad);
    assert_removed(&app, &old_map);
    let (_, player) = assert_road(&mut app, RoadId::ExampleRoad);
    let spawn = app.world().get::<Position>(player).unwrap().0;
    advance(&mut app, 3);

    // A real shop sensor connects keyboard input, collision detection, and its observer.
    let shop = {
        let world = app.world_mut();
        world
            .query::<(Entity, &BuildingType)>()
            .iter(world)
            .find_map(|(entity, kind)| (*kind == BuildingType::ConvenienceStore).then_some(entity))
            .unwrap()
    };
    let price =
        app.world().resource::<BuildingValues>().convenience_stores[&ConvenienceKind::Small].price;
    let balance = price * 3 + 1;
    app.world_mut().resource_mut::<Money>().0 = balance;
    app.world_mut().resource_mut::<Stamina>().0 = 1.0;
    press_e(&mut app);
    assert_eq!(app.world().resource::<Money>().0, balance);
    let center = app
        .world()
        .get::<GlobalTransform>(shop)
        .unwrap()
        .translation();
    teleport(&mut app, player, center);
    assert!(
        app.world()
            .get::<CollidingEntities>(shop)
            .unwrap()
            .contains(&player)
    );
    assert_eq!(
        app.world().resource::<Money>().0,
        balance,
        "entering while E is held does nothing"
    );
    press_e(&mut app);
    let after_purchase = app.world().resource::<Money>().0;
    assert!(after_purchase < balance);
    assert!(app.world().resource::<Stamina>().0 > 1.0);
    advance(&mut app, 3);
    assert_eq!(
        app.world().resource::<Money>().0,
        after_purchase,
        "holding E does not purchase again"
    );

    app.world_mut().resource_mut::<Money>().0 = 37;
    app.world_mut().resource_mut::<Stamina>().0 = 42.0;
    app.world_mut().resource_mut::<HasHelmet>().0 = true;
    app.world_mut().resource_mut::<GameSettings>().commute_limit = 123.0;
    for (destination, choices) in [
        (RoadId::MarketRoad, &[RoadId::MarketRoad, RoadId::Court][..]),
        (RoadId::Court, &[RoadId::Court][..]),
    ] {
        let road = single::<SceneRoot>(&mut app);
        let player = single::<Player>(&mut app);
        let old_road = scene_entities(app.world(), road);
        let before = app.world().resource::<CommuteTime>().elapsed;
        advance(&mut app, 3);
        assert_commute(&app, CommuteState::Running);
        assert!(app.world().resource::<CommuteTime>().elapsed > before);

        let point = single::<SwitchPoint>(&mut app);
        let center = app
            .world()
            .get::<GlobalTransform>(point)
            .unwrap()
            .translation();
        teleport(&mut app, player, center);
        assert!(
            app.world()
                .get::<CollidingEntities>(point)
                .unwrap()
                .contains(&player)
        );
        press_e(&mut app);
        app.update();
        assert_removed(&app, &old_road);
        let map = assert_map(&mut app, choices);
        let old_map = scene_entities(app.world(), map);
        assert_commute(&app, CommuteState::Paused);
        let paused_at = app.world().resource::<CommuteTime>().elapsed;
        advance(&mut app, 4);
        assert_eq!(app.world().resource::<CommuteTime>().elapsed, paused_at);

        activate(&mut app, destination);
        assert_removed(&app, &old_map);
        let (_, next_player) = assert_road(&mut app, destination);
        assert_ne!(next_player, player);
        assert_eq!(app.world().get::<Position>(next_player).unwrap().0, spawn);
        assert_eq!(
            app.world().get::<LinearVelocity>(next_player).unwrap().0,
            Vec2::ZERO
        );
        advance(&mut app, 3);
        if destination == RoadId::Court {
            assert_commute(&app, CommuteState::Paused);
            assert_eq!(app.world().resource::<CommuteTime>().elapsed, paused_at);
        } else {
            assert_commute(&app, CommuteState::Running);
            assert!(app.world().resource::<CommuteTime>().elapsed > paused_at);
        }
        assert!(app.world().get_entity(camera).is_ok());
        assert!(app.world().get_entity(unrelated).is_ok());
        assert_eq!(app.world().resource::<Money>().0, 37);
        assert_eq!(app.world().resource::<Stamina>().0, 42.0);
        assert!(app.world().resource::<HasHelmet>().0);
        assert_eq!(app.world().resource::<GameSettings>().commute_limit, 123.0);
    }
}

#[test]
fn missing_empty_and_logical_connections_offer_no_travel() {
    let mut app = navigation_app();
    let road = single::<SceneRoot>(&mut app);
    let player = single::<Player>(&mut app);
    let point = single::<SwitchPoint>(&mut app);
    for connections in [None, Some(vec![]), Some(vec![RoadId::Start])] {
        let mut graph = app.world_mut().resource_mut::<RoadGraph>();
        match connections {
            Some(connections) => {
                graph.roads.insert(RoadId::ExampleRoad, connections);
            }
            None => {
                graph.roads.remove(&RoadId::ExampleRoad);
            }
        }
        assert!(destinations(&graph, RoadId::ExampleRoad).is_empty());
        open_destinations(app.world_mut(), point, player);
        assert_eq!(single::<SceneRoot>(&mut app), road);
        assert_eq!(single::<Player>(&mut app), player);
    }
}

#[test]
fn navigation_rejects_invalid_actors_and_destinations() {
    let mut app = navigation_app();
    let road = single::<SceneRoot>(&mut app);
    let player = single::<Player>(&mut app);
    let point = single::<SwitchPoint>(&mut app);
    let foreign_root = app.world_mut().spawn(SceneRoot).id();
    let foreign_player = app
        .world_mut()
        .spawn((Player::default(), ChildOf(foreign_root)))
        .id();
    let orphan = app
        .world_mut()
        .spawn(TravelButtonAction(RoadId::Court))
        .id();
    select_destination(app.world_mut(), orphan);
    for (target, actor) in [
        (point, Entity::PLACEHOLDER),
        (point, foreign_player),
        (road, player),
    ] {
        open_destinations(app.world_mut(), target, actor);
    }
    assert_eq!(app.world().get::<RoadId>(road), Some(&RoadId::ExampleRoad));
    assert!(app.world().get_entity(player).is_ok());
    assert_absent::<TravelUiRoot>(&mut app);
    app.world_mut().despawn(foreign_root);
    app.world_mut().despawn(orphan);

    open_destinations(app.world_mut(), point, player);
    let map = assert_map(&mut app, &[RoadId::MarketRoad, RoadId::Court]);
    let button = destination_button(&mut app, RoadId::MarketRoad);
    for (destination, choices) in [
        (RoadId::ExampleRoad, vec![RoadId::MarketRoad, RoadId::Court]),
        (RoadId::Start, vec![RoadId::Start]),
    ] {
        app.world_mut()
            .get_mut::<TravelButtonAction>(button)
            .unwrap()
            .0 = destination;
        app.world_mut().get_mut::<TravelUiRoot>(map).unwrap().0 = choices;
        select_destination(app.world_mut(), button);
        assert_eq!(single::<SceneRoot>(&mut app), map);
        assert_absent::<Player>(&mut app);
    }
}

#[test]
fn duplicate_and_stale_events_cannot_navigate_a_replacement_scene() {
    let mut app = navigation_app();
    let player = single::<Player>(&mut app);
    let point = single::<SwitchPoint>(&mut app);
    for _ in 0..3 {
        app.world_mut().commands().trigger(InteractRequested {
            entity: point,
            player,
        });
    }
    app.world_mut().flush();
    let map = assert_map(&mut app, &[RoadId::MarketRoad, RoadId::Court]);
    let market = destination_button(&mut app, RoadId::MarketRoad);
    let stale_court = destination_button(&mut app, RoadId::Court);
    for button in [market, stale_court, market] {
        app.world_mut()
            .commands()
            .trigger(Activate { entity: button });
    }
    app.world_mut().flush();
    let (road, next_player) = assert_road(&mut app, RoadId::MarketRoad);
    assert!(app.world().get_entity(map).is_err());
    open_destinations(app.world_mut(), point, player);
    select_destination(app.world_mut(), stale_court);
    assert_eq!(single::<SceneRoot>(&mut app), road);

    let next_point = single::<SwitchPoint>(&mut app);
    open_destinations(app.world_mut(), next_point, next_player);
    let next_map = assert_map(&mut app, &[RoadId::Court]);
    select_destination(app.world_mut(), stale_court);
    assert_eq!(single::<SceneRoot>(&mut app), next_map);
    activate(&mut app, RoadId::Court);
    assert_road(&mut app, RoadId::Court);
}
