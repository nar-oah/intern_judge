use super::{
    InitialRoad, LevelPlugin, PausedInteractRequested, RoadEffect, RoadEffects, RoadId, RoadUnlocks,
    SceneRoot, SwitchPoint,
    bill::{WeeklyBill, confirm_bill, show_bill},
    day::{DayEndRequested, begin_next_day, start_day},
    navigation::{open_destinations, select_destination},
    scene::{
        effect::{ConstructionDetour, FallingObject},
        ending::InternshipEnded,
    },
    ui::{TravelButtonAction, TravelUiRoot, UiAction},
};
use crate::{
    config::{GameSettings, GameState, RoadGraph},
    game::{
        CommuteState, CommuteTime, GamePlugin, HasHelmet, Money, Stamina, calendar::GameCalendar,
    },
    player::{Player, PlayerAction, PlayerPlugin, Stunned},
    test_support::{advance, finish_app, headless_app},
};
use avian2d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_replique::prelude::{DialogueFinished, DialogueRunner};
use leafwing_input_manager::prelude::{ActionState, Buttonlike, InputMap};
use std::time::{Duration, Instant};

fn day_app(total_days: u32) -> App {
    configured_day_app(total_days, |_| {})
}

fn configured_day_app(total_days: u32, configure: impl FnOnce(&mut App)) -> App {
    let mut app = headless_app(Vec2::ZERO);
    app.world_mut().resource_mut::<GameSettings>().total_days = total_days;
    configure(&mut app);
    app.init_asset::<Font>()
        .init_asset_loader::<bevy::text::FontLoader>()
        .add_plugins((GamePlugin, PlayerPlugin, LevelPlugin));
    finish_app(&mut app);
    app
}

fn weekday_app(total_days: u32) -> App {
    let mut app = day_app(total_days);
    app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 2;
    start_day(app.world_mut());
    app.update();
    app
}

fn shipped_road_app() -> App {
    let mut app = headless_app(Vec2::ZERO);
    app.insert_state(GameState::MainMenu)
        .init_asset::<Font>()
        .init_asset_loader::<bevy::text::FontLoader>()
        .add_plugins((
            RonAssetPlugin::<RoadGraph>::new(&["graph.ron"]),
            GamePlugin,
            PlayerPlugin,
            LevelPlugin,
        ));
    let handle = app
        .world()
        .resource::<AssetServer>()
        .load::<RoadGraph>("roads.graph.ron");
    finish_app(&mut app);
    let deadline = Instant::now() + Duration::from_secs(10);
    let graph = loop {
        if let Some(graph) = app.world().resource::<Assets<RoadGraph>>().get(&handle) {
            break graph.clone();
        }
        assert!(Instant::now() < deadline, "shipped road graph did not load");
        app.update();
        std::thread::sleep(Duration::from_millis(1));
    };
    app.insert_resource(graph);
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    advance(&mut app, 2);
    app
}

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

fn assert_road(app: &mut App, road: RoadId) -> (Entity, Entity) {
    let root = single::<SceneRoot>(app);
    let player = single::<Player>(app);
    assert_eq!(app.world().get::<RoadId>(root), Some(&road));
    assert_eq!(app.world().get::<ChildOf>(player).unwrap().parent(), root);
    assert_absent::<InternshipEnded>(app);
    assert_absent::<TravelUiRoot>(app);
    (root, player)
}

fn assert_commute(app: &App, expected: CommuteState) {
    assert_eq!(
        *app.world().resource::<State<CommuteState>>().get(),
        expected
    );
}

fn assert_day_scene(app: &mut App) {
    if app.world().resource::<GameCalendar>().is_weekend() {
        single::<TravelUiRoot>(app);
        assert_absent::<Player>(app);
        assert_commute(app, CommuteState::Paused);
    } else {
        assert_road(app, RoadId::ExampleRoad);
        assert_commute(app, CommuteState::Running);
    }
}

fn calendar_hud(app: &mut App) -> (Entity, Entity, String) {
    let world = app.world_mut();
    let (text_entity, text) = world
        .query::<(Entity, &Text)>()
        .iter(world)
        .find_map(|(entity, text)| {
            text.0
                .starts_with("Days left: ")
                .then_some((entity, text.0.clone()))
        })
        .expect("the calendar HUD contains its date text");
    let root = world.get::<ChildOf>(text_entity).unwrap().parent();
    assert!(world.get::<ChildOf>(root).is_none());
    assert!(world.get::<SceneRoot>(root).is_none());
    let node = world.get::<Node>(root).unwrap();
    assert_eq!(node.left, px(16));
    assert_eq!(node.top, px(16));
    (root, text_entity, text)
}

#[test]
fn new_game_starts_with_the_weekend_unlock_map_and_resets_unlock_progress() {
    let mut app = configured_day_app(28, |app| {
        app.world_mut()
            .resource_mut::<RoadGraph>()
            .initial_unlocks
            .insert(RoadId::MarketRoad, false);
    });
    advance(&mut app, 2);

    for restart in [false, true] {
        if restart {
            app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 5;
            app.world_mut()
                .resource_mut::<RoadUnlocks>()
                .0
                .insert(RoadId::MarketRoad, true);
            app.world_mut()
                .resource_mut::<NextState<CommuteState>>()
                .set(CommuteState::Running);
            app.world_mut()
                .resource_mut::<NextState<GameState>>()
                .set(GameState::MainMenu);
            app.update();
            app.world_mut()
                .resource_mut::<NextState<GameState>>()
                .set(GameState::Playing);
            advance(&mut app, 2);
        }

        let root = single::<TravelUiRoot>(&mut app);
        assert_eq!(
            app.world().get::<TravelUiRoot>(root).unwrap().0,
            [RoadId::MarketRoad]
        );
        assert_absent::<Player>(&mut app);
        assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 0);
        assert_eq!(
            app.world().resource::<RoadUnlocks>().0,
            app.world().resource::<RoadGraph>().initial_unlocks
        );
        assert_commute(&app, CommuteState::Paused);
        let paused_at = app.world().resource::<CommuteTime>().elapsed;
        advance(&mut app, 4);
        assert_eq!(app.world().resource::<CommuteTime>().elapsed, paused_at);
    }
}

#[test]
fn weekend_unlock_allows_movement_and_ends_into_the_next_days_map_then_workday() {
    let mut app = configured_day_app(28, |app| {
        let mut graph = app.world_mut().resource_mut::<RoadGraph>();
        graph.initial_unlocks.insert(RoadId::MarketRoad, false);
        graph.initial_unlocks.insert(RoadId::EmptyRoad1, false);
        graph
            .roads
            .get_mut(&RoadId::MarketRoad)
            .unwrap()
            .push(RoadId::EmptyRoad1);
        graph
            .roads
            .insert(RoadId::EmptyRoad1, vec![RoadId::MarketRoad]);
    });
    let map = single::<TravelUiRoot>(&mut app);
    let button = single::<TravelButtonAction>(&mut app);
    select_destination(app.world_mut(), button);

    assert!(app.world().resource::<RoadUnlocks>().0[&RoadId::MarketRoad]);
    assert!(app.world().get_entity(map).is_err());
    let (_, player) = assert_road(&mut app, RoadId::MarketRoad);
    advance(&mut app, 2);
    assert_commute(&app, CommuteState::Paused);
    let paused_at = app.world().resource::<CommuteTime>().elapsed;
    KeyCode::KeyD.press(app.world_mut());
    advance(&mut app, 2);
    assert!(app.world().get::<LinearVelocity>(player).unwrap().x > 0.0);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, paused_at);

    let unlocks = app.world().resource::<RoadUnlocks>().0.clone();
    let point = single::<SwitchPoint>(&mut app);
    app.world_mut().trigger(PausedInteractRequested {
        entity: point,
        player,
    });
    advance(&mut app, 2);

    assert!(app.world().get_entity(player).is_err());
    assert_absent::<Player>(&mut app);
    let map = single::<TravelUiRoot>(&mut app);
    assert_eq!(
        app.world().get::<TravelUiRoot>(map).unwrap().0,
        [RoadId::EmptyRoad1]
    );
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 1);
    assert_eq!(app.world().resource::<RoadUnlocks>().0, unlocks);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.0);
    assert_commute(&app, CommuteState::Paused);

    let button = single::<TravelButtonAction>(&mut app);
    select_destination(app.world_mut(), button);
    let (_, player) = assert_road(&mut app, RoadId::EmptyRoad1);
    advance(&mut app, 2);
    assert!(app.world().get_entity(map).is_err());
    assert_commute(&app, CommuteState::Paused);
    assert!(app.world().resource::<RoadUnlocks>().0[&RoadId::EmptyRoad1]);
    let unlocks = app.world().resource::<RoadUnlocks>().0.clone();
    let point = single::<SwitchPoint>(&mut app);
    app.world_mut().trigger(PausedInteractRequested {
        entity: point,
        player,
    });
    advance(&mut app, 2);

    assert!(app.world().get_entity(player).is_err());
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 2);
    assert_eq!(app.world().resource::<RoadUnlocks>().0, unlocks);
    assert_day_scene(&mut app);
    assert!(app.world().resource::<CommuteTime>().elapsed > 0.0);
}

#[test]
fn shipped_roads_support_all_eight_weekend_unlocks_and_exit_into_the_next_day() {
    let mut app = shipped_road_app();
    let mut weekend_roads = Vec::new();

    for day in 0..28 {
        assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, day);
        let weekend = app.world().resource::<GameCalendar>().is_weekend();
        let friday = app.world().resource::<GameCalendar>().weekday() == 5;
        let destination = if weekend {
            let map = single::<TravelUiRoot>(&mut app);
            let choices = &app.world().get::<TravelUiRoot>(map).unwrap().0;
            let &road = choices
                .first()
                .expect("every weekend must offer a road to unlock");
            assert!(!app.world().resource::<RoadUnlocks>().0[&road]);
            assert!(!weekend_roads.contains(&road));
            weekend_roads.push(road);
            road
        } else {
            let player = single::<Player>(&mut app);
            let point = single::<SwitchPoint>(&mut app);
            open_destinations(app.world_mut(), point, player);
            RoadId::Court
        };
        let button = {
            let world = app.world_mut();
            world
                .query::<(Entity, &TravelButtonAction)>()
                .iter(world)
                .find_map(|(entity, action)| (action.0 == destination).then_some(entity))
                .expect("the day's destination is selectable")
        };
        select_destination(app.world_mut(), button);
        assert!(app.world().resource::<RoadUnlocks>().0[&destination]);
        let (_, player) = assert_road(&mut app, destination);
        advance(&mut app, 2);
        assert_commute(&app, CommuteState::Paused);
        let paused_at = app.world().resource::<CommuteTime>().elapsed;
        advance(&mut app, 2);
        assert_eq!(app.world().resource::<CommuteTime>().elapsed, paused_at);

        let unlocks = app.world().resource::<RoadUnlocks>().0.clone();
        if weekend {
            let point = single::<SwitchPoint>(&mut app);
            app.world_mut().trigger(PausedInteractRequested {
                entity: point,
                player,
            });
        } else {
            app.world_mut().insert_resource(DayEndRequested);
        }
        app.update();
        assert!(app.world().get_entity(player).is_err());
        if friday {
            single::<WeeklyBill>(&mut app);
            assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, day);
            KeyCode::KeyE.release(app.world_mut());
            app.update();
            KeyCode::KeyE.press(app.world_mut());
            app.update();
            KeyCode::KeyE.release(app.world_mut());
        }
        advance(&mut app, 2);
        assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, day + 1);
        assert_eq!(app.world().resource::<RoadUnlocks>().0, unlocks);
    }

    assert_eq!(weekend_roads.len(), 8);
    assert_eq!(weekend_roads[0], RoadId::MarketRoad);
    weekend_roads.sort_unstable();
    weekend_roads.dedup();
    assert_eq!(weekend_roads.len(), 8);
    single::<InternshipEnded>(&mut app);
}

#[test]
fn next_day_resets_the_scene_and_daily_values_while_preserving_session_progress() {
    let mut app = weekday_app(28);
    let camera = app.world_mut().spawn(Camera2d).id();
    let unrelated = app.world_mut().spawn_empty().id();
    let (old_root, old_player) = assert_road(&mut app, RoadId::ExampleRoad);
    let old_scene = scene_entities(app.world(), old_root);
    let spawn = app.world().get::<Position>(old_player).unwrap().0;
    app.world_mut().resource_mut::<InitialRoad>().0 = RoadId::MarketRoad;
    app.world_mut().resource_mut::<Money>().0 = 41;
    app.world_mut().resource_mut::<HasHelmet>().0 = true;
    app.world_mut().resource_mut::<RoadEffects>().0 = [
        (RoadId::ExampleRoad, vec![RoadEffect::FallingObject]),
        (
            RoadId::MarketRoad,
            vec![RoadEffect::ConstructionDetour, RoadEffect::FallingObject],
        ),
    ]
    .into();
    let effects = app.world().resource::<RoadEffects>().0.clone();
    app.world_mut()
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Paused);
    app.update();
    app.world_mut().resource_mut::<Stamina>().0 = 3.0;
    app.world_mut().resource_mut::<CommuteTime>().elapsed = 51.0;
    app.world_mut()
        .get_mut::<LinearVelocity>(old_player)
        .unwrap()
        .0 = Vec2::new(123.0, -54.0);
    app.world_mut().get_mut::<Position>(old_player).unwrap().0 = Vec2::new(500.0, 200.0);
    app.world_mut()
        .entity_mut(old_player)
        .insert(Stunned { remaining: 9.0 });
    app.world_mut()
        .get_mut::<ActionState<PlayerAction>>(old_player)
        .unwrap()
        .press(&PlayerAction::Right);

    begin_next_day(app.world_mut());

    assert_removed(&app, &old_scene);
    let (root, player) = assert_road(&mut app, RoadId::MarketRoad);
    assert_ne!(root, old_root);
    assert_ne!(player, old_player);
    assert_eq!(app.world().get::<Position>(player).unwrap().0, spawn);
    assert_eq!(
        app.world().get::<LinearVelocity>(player).unwrap().0,
        Vec2::ZERO
    );
    assert!(app.world().get::<Stunned>(player).is_none());
    assert!(
        !app.world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap()
            .pressed(&PlayerAction::Right)
    );
    for effect in [
        single::<FallingObject>(&mut app),
        single::<ConstructionDetour>(&mut app),
    ] {
        assert_eq!(app.world().get::<ChildOf>(effect).unwrap().parent(), root);
    }
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 3);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.0);
    assert_eq!(app.world().resource::<Stamina>().0, Stamina::MAX);
    assert_eq!(app.world().resource::<Money>().0, 41);
    assert!(app.world().resource::<HasHelmet>().0);
    assert_eq!(app.world().resource::<RoadEffects>().0, effects);
    assert!(app.world().get_entity(camera).is_ok());
    assert!(app.world().get_entity(unrelated).is_ok());
    assert_eq!(
        *app.world().resource::<State<GameState>>().get(),
        GameState::Playing
    );

    advance(&mut app, 2);
    assert_commute(&app, CommuteState::Running);
    assert!(app.world().resource::<CommuteTime>().elapsed > 0.0);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 3);
    assert_eq!(app.world().resource::<Money>().0, 41);
    assert!(app.world().resource::<HasHelmet>().0);
    assert_eq!(app.world().resource::<RoadEffects>().0, effects);
}

#[test]
fn requested_day_end_waits_for_dialogue_cleanup_and_runs_only_once() {
    let mut app = weekday_app(28);
    let (root, player) = assert_road(&mut app, RoadId::ExampleRoad);
    app.world_mut()
        .entity_mut(player)
        .insert(DialogueRunner::new(Handle::default()));
    app.world_mut().insert_resource(DayEndRequested);

    advance(&mut app, 2);

    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 2);
    assert_eq!(single::<SceneRoot>(&mut app), root);
    assert!(app.world().get::<DialogueRunner>(player).is_some());
    assert!(app.world().contains_resource::<DayEndRequested>());

    app.world_mut()
        .write_message(DialogueFinished { runner: player });
    app.update();

    assert!(app.world().get_entity(root).is_err());
    assert!(app.world().get_entity(player).is_err());
    assert_absent::<DialogueRunner>(&mut app);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 3);
    assert!(!app.world().contains_resource::<DayEndRequested>());
    let (new_root, _) = assert_road(&mut app, RoadId::ExampleRoad);
    advance(&mut app, 3);
    assert_eq!(single::<SceneRoot>(&mut app), new_root);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 3);
}

#[test]
fn friday_bill_waits_for_dialogue_cleanup_and_a_fresh_confirm_before_changing_money_or_date() {
    let mut app = weekday_app(28);
    let (root, player) = assert_road(&mut app, RoadId::ExampleRoad);
    let old_scene = scene_entities(app.world(), root);
    let (hud, text, _) = calendar_hud(&mut app);
    app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 6;
    app.world_mut().resource_mut::<Money>().0 = 12;
    app.world_mut().resource_mut::<HasHelmet>().0 = true;
    app.world_mut()
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Paused);
    app.update();
    app.world_mut().resource_mut::<CommuteTime>().elapsed = 17.0;
    app.world_mut().resource_mut::<Stamina>().0 = 4.0;
    app.world_mut()
        .entity_mut(player)
        .insert(DialogueRunner::new(Handle::default()));
    app.world_mut().insert_resource(DayEndRequested);
    KeyCode::KeyE.press(app.world_mut());
    advance(&mut app, 2);

    assert_absent::<WeeklyBill>(&mut app);
    assert_eq!(single::<SceneRoot>(&mut app), root);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 6);
    assert!(app.world().contains_resource::<DayEndRequested>());

    app.world_mut()
        .write_message(DialogueFinished { runner: player });
    advance(&mut app, 4);

    assert_removed(&app, &old_scene);
    assert_absent::<DialogueRunner>(&mut app);
    assert_absent::<Player>(&mut app);
    let bill = single::<WeeklyBill>(&mut app);
    assert_eq!(single::<SceneRoot>(&mut app), bill);
    assert!(app.world().get::<InputMap<UiAction>>(bill).is_some());
    assert!(app.world().get::<ActionState<UiAction>>(bill).is_some());
    assert!(!app.world().contains_resource::<DayEndRequested>());
    assert_commute(&app, CommuteState::Paused);
    assert_eq!(app.world().resource::<Money>().0, 12);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 6);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 17.0);
    assert_eq!(app.world().resource::<Stamina>().0, 4.0);
    let (new_hud, new_text, date) = calendar_hud(&mut app);
    assert_eq!((new_hud, new_text), (hud, text));
    assert!(date.contains("Friday"));
    let bill_entities = scene_entities(app.world(), bill);
    let summary = bill_entities
        .iter()
        .find_map(|&entity| app.world().get::<Text>(entity))
        .unwrap();
    for expected in ["当前余额：12", "本周工资：31", "房租：47", "结算后余额：-4"]
    {
        assert!(summary.0.contains(expected));
    }

    app.world_mut().insert_resource(DayEndRequested);
    app.update();
    assert_eq!(single::<WeeklyBill>(&mut app), bill);
    assert_eq!(app.world().resource::<Money>().0, 12);
    assert!(!app.world().contains_resource::<DayEndRequested>());
    {
        let mut settings = app.world_mut().resource_mut::<GameSettings>();
        settings.weekly_salary = 103;
        settings.weekly_rent = 7;
    }
    KeyCode::KeyE.release(app.world_mut());
    app.update();
    KeyCode::KeyE.press(app.world_mut());
    app.update();

    assert_removed(&app, &bill_entities);
    assert_absent::<WeeklyBill>(&mut app);
    single::<TravelUiRoot>(&mut app);
    assert_absent::<Player>(&mut app);
    assert_eq!(app.world().resource::<Money>().0, -4);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 7);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.0);
    assert_eq!(app.world().resource::<Stamina>().0, Stamina::MAX);
    assert!(app.world().resource::<HasHelmet>().0);
    confirm_bill(app.world_mut());
    advance(&mut app, 4);
    assert_eq!(app.world().resource::<Money>().0, -4);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 7);
    assert_commute(&app, CommuteState::Paused);
}

#[test]
fn day_changes_and_bills_replace_all_scene_roots_and_keep_unrelated_entities() {
    for billing in [false, true] {
        for root_count in [0, 2] {
            let mut app = weekday_app(28);
            let old_root = single::<SceneRoot>(&mut app);
            app.world_mut().despawn(old_root);
            let camera = app.world_mut().spawn(Camera2d).id();
            let unrelated = app.world_mut().spawn_empty().id();
            let (hud, text, _) = calendar_hud(&mut app);
            let mut previous = Vec::new();
            for _ in 0..root_count {
                let root = app.world_mut().spawn(SceneRoot).id();
                let child = app.world_mut().spawn(ChildOf(root)).id();
                previous.extend([root, child]);
            }

            if billing {
                show_bill(app.world_mut());
            } else {
                begin_next_day(app.world_mut());
            }

            assert_removed(&app, &previous);
            let root = single::<SceneRoot>(&mut app);
            if billing {
                assert_eq!(single::<WeeklyBill>(&mut app), root);
                assert_absent::<Player>(&mut app);
                assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 2);
            } else {
                assert_road(&mut app, RoadId::ExampleRoad);
                assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 3);
            }
            assert!(app.world().get_entity(camera).is_ok());
            assert!(app.world().get_entity(unrelated).is_ok());
            let (new_hud, new_text, _) = calendar_hud(&mut app);
            assert_eq!((new_hud, new_text), (hud, text));
        }
    }
}

#[test]
fn each_friday_settles_once_and_other_days_advance_without_billing() {
    let mut app = day_app(28);
    let mut balance = app.world().resource::<Money>().0;
    for day in 0..28 {
        let friday = app.world().resource::<GameCalendar>().weekday() == 5;
        app.world_mut().insert_resource(DayEndRequested);
        app.update();
        if friday {
            single::<WeeklyBill>(&mut app);
            assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, day);
            assert_eq!(app.world().resource::<Money>().0, balance);
            KeyCode::KeyE.release(app.world_mut());
            app.update();
            KeyCode::KeyE.press(app.world_mut());
            app.update();
            let settings = app.world().resource::<GameSettings>();
            balance += settings.weekly_salary - settings.weekly_rent;
        }
        advance(&mut app, 2);
        assert_absent::<WeeklyBill>(&mut app);
        assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, day + 1);
        assert_eq!(app.world().resource::<Money>().0, balance);
    }
    single::<InternshipEnded>(&mut app);
    app.world_mut().insert_resource(DayEndRequested);
    advance(&mut app, 3);
    assert_absent::<WeeklyBill>(&mut app);
    assert_eq!(app.world().resource::<Money>().0, balance);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 28);
}

#[test]
fn calendar_hud_survives_road_travel_and_refreshes_after_the_next_day() {
    let mut app = weekday_app(28);
    let (hud, text, initial_date) = calendar_hud(&mut app);
    let player = single::<Player>(&mut app);
    let point = single::<SwitchPoint>(&mut app);
    open_destinations(app.world_mut(), point, player);
    assert_absent::<Player>(&mut app);
    assert_eq!(calendar_hud(&mut app), (hud, text, initial_date.clone()));
    let button = {
        let world = app.world_mut();
        world
            .query::<(Entity, &TravelButtonAction)>()
            .iter(world)
            .find_map(|(entity, action)| (action.0 == RoadId::MarketRoad).then_some(entity))
            .unwrap()
    };
    select_destination(app.world_mut(), button);
    app.update();
    assert_road(&mut app, RoadId::MarketRoad);
    assert_eq!(calendar_hud(&mut app), (hud, text, initial_date.clone()));

    begin_next_day(app.world_mut());
    app.update();

    let (new_hud, new_text, next_date) = calendar_hud(&mut app);
    assert_eq!(new_hud, hud);
    assert_eq!(new_text, text);
    assert_ne!(next_date, initial_date);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 3);
    assert_road(&mut app, RoadId::ExampleRoad);
}

#[test]
fn ending_the_last_day_shows_the_end_screen_and_cannot_restart_the_commute() {
    for total_days in [1, 3, 28] {
        let mut app = day_app(total_days);
        app.world_mut().resource_mut::<Money>().0 = 59;
        app.world_mut().resource_mut::<HasHelmet>().0 = true;
        for elapsed_days in 1..total_days {
            let root = single::<SceneRoot>(&mut app);
            begin_next_day(app.world_mut());
            app.update();
            assert!(app.world().get_entity(root).is_err());
            assert_day_scene(&mut app);
            assert_eq!(
                app.world().resource::<GameCalendar>().elapsed_days,
                elapsed_days
            );
            assert_eq!(
                app.world().resource::<GameCalendar>().remaining_days(),
                total_days - elapsed_days
            );
        }
        let old_root = single::<SceneRoot>(&mut app);
        let old_scene = scene_entities(app.world(), old_root);
        app.world_mut().resource_mut::<CommuteTime>().elapsed = 17.0;
        app.world_mut().resource_mut::<Stamina>().0 = 2.0;

        begin_next_day(app.world_mut());
        app.update();

        assert_removed(&app, &old_scene);
        let root = single::<SceneRoot>(&mut app);
        assert_eq!(single::<InternshipEnded>(&mut app), root);
        assert_absent::<Player>(&mut app);
        assert_absent::<RoadId>(&mut app);
        assert_absent::<SwitchPoint>(&mut app);
        let calendar = app.world().resource::<GameCalendar>();
        assert_eq!(calendar.elapsed_days, total_days);
        assert_eq!(calendar.remaining_days(), 0);
        assert!(calendar.is_finished());
        assert_eq!(app.world().resource::<Stamina>().0, Stamina::MAX);
        assert_eq!(app.world().resource::<Money>().0, 59);
        assert!(app.world().resource::<HasHelmet>().0);
        assert_commute(&app, CommuteState::Paused);
        assert_eq!(
            *app.world().resource::<State<GameState>>().get(),
            GameState::Playing
        );
        let world = app.world_mut();
        assert!(
            world
                .query::<&Text>()
                .iter(world)
                .any(|text| text.0 == "实习结束")
        );
        assert_eq!(world.get::<Node>(root).unwrap().width, percent(100));
        assert_eq!(world.get::<Node>(root).unwrap().height, percent(100));

        begin_next_day(app.world_mut());
        app.world_mut().insert_resource(DayEndRequested);
        advance(&mut app, 4);

        assert_eq!(single::<SceneRoot>(&mut app), root);
        assert_absent::<Player>(&mut app);
        assert_absent::<RoadId>(&mut app);
        assert_eq!(
            app.world().resource::<GameCalendar>().elapsed_days,
            total_days
        );
        assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.0);
        assert_commute(&app, CommuteState::Paused);
        assert!(!app.world().contains_resource::<DayEndRequested>());
    }
}
