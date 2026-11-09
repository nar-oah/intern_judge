use super::support::assert_day_scene;
use crate::{
    config::{GameSettings, GameState, RoadGraph},
    game::{CommuteState, CommuteTime, GamePlugin, Money, calendar::GameCalendar},
    level::{
        LevelPlugin, PausedInteractRequested, RoadId, RoadUnlocks, SwitchPoint,
        bill::DailyBill,
        day::EndDay,
        scene::ending::InternshipEnded,
        tests::support::{
            assert_commute, assert_road, assert_today_roads, configured_day_app, destination_button,
        },
        tests::support::{open_destinations, select_destination},
        ui::{TravelButtonAction, TravelUiRoot},
    },
    player::{Player, PlayerPlugin},
    test_support::{advance, assert_absent, finish_app, headless_app, press_e, single},
};
use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use leafwing_input_manager::prelude::Buttonlike;
use std::time::{Duration, Instant};

fn shipped_road_app() -> App {
    let mut app = headless_app(Vec2::ZERO);
    app.insert_state(GameState::MainMenu).add_plugins((
        RonAssetPlugin::<RoadGraph>::new(&["graph.ron"]),
        RonAssetPlugin::<GameSettings>::new(&["settings.ron"]),
        GamePlugin,
        PlayerPlugin,
        LevelPlugin,
    ));
    let handle = app
        .world()
        .resource::<AssetServer>()
        .load::<RoadGraph>("roads.graph.ron");
    let settings_handle = app
        .world()
        .resource::<AssetServer>()
        .load::<GameSettings>("game.settings.ron");
    finish_app(&mut app);
    let deadline = Instant::now() + Duration::from_secs(10);
    let (graph, settings) = loop {
        if let (Some(graph), Some(settings)) = (
            app.world().resource::<Assets<RoadGraph>>().get(&handle),
            app.world()
                .resource::<Assets<GameSettings>>()
                .get(&settings_handle),
        ) {
            break (graph.clone(), settings.clone());
        }
        assert!(
            Instant::now() < deadline,
            "shipped road graph and settings did not load"
        );
        app.update();
        std::thread::sleep(Duration::from_millis(1));
    };
    app.insert_resource(graph).insert_resource(settings);
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    advance(&mut app, 2);
    app
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
    assert_today_roads(&app, &[]);
    let button = single::<TravelButtonAction>(&mut app);
    select_destination(app.world_mut(), button);

    assert!(app.world().resource::<RoadUnlocks>().0[&RoadId::MarketRoad]);
    assert!(app.world().get_entity(map).is_err());
    let (_, player) = assert_road(&mut app, RoadId::MarketRoad);
    assert_today_roads(&app, &[RoadId::MarketRoad]);
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
    single::<DailyBill>(&mut app);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 0);
    press_e(&mut app);
    advance(&mut app, 2);
    let map = single::<TravelUiRoot>(&mut app);
    assert_today_roads(&app, &[]);
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
    assert_today_roads(&app, &[RoadId::EmptyRoad1]);
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
    single::<DailyBill>(&mut app);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 1);
    press_e(&mut app);
    advance(&mut app, 2);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 2);
    assert_eq!(app.world().resource::<RoadUnlocks>().0, unlocks);
    assert_day_scene(&mut app);
    assert!(app.world().resource::<CommuteTime>().elapsed > 0.0);
}

#[test]
fn shipped_month_pays_daily_unlocks_eight_roads_and_charges_rent_only_fridays() {
    let mut app = shipped_road_app();
    let mut weekend_roads = Vec::new();
    let mut balance = app.world().resource::<Money>().0;
    assert_eq!(balance, 100);

    for day in 0..28 {
        assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, day);
        let weekend = app.world().resource::<GameCalendar>().is_weekend();
        // Day zero is Saturday, so rent is due on days 6, 13, 20 and 27.
        let friday = day % 7 == 6;
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
            let point = single::<SwitchPoint>(&mut app);
            open_destinations(app.world_mut(), point);
            RoadId::Court
        };
        let button = destination_button(&mut app, destination);
        select_destination(app.world_mut(), button);
        assert!(app.world().resource::<RoadUnlocks>().0[&destination]);
        let (_, player) = assert_road(&mut app, destination);
        advance(&mut app, 2);
        assert_commute(&app, CommuteState::Paused);
        let paused_at = app.world().resource::<CommuteTime>().elapsed;
        advance(&mut app, 2);
        assert_eq!(app.world().resource::<CommuteTime>().elapsed, paused_at);
        if !weekend {
            app.world_mut().resource_mut::<CommuteTime>().elapsed = 50.0;
        }

        let unlocks = app.world().resource::<RoadUnlocks>().0.clone();
        if weekend {
            let point = single::<SwitchPoint>(&mut app);
            app.world_mut().trigger(PausedInteractRequested {
                entity: point,
                player,
            });
        } else {
            app.world_mut().trigger(EndDay);
        }
        app.update();
        assert!(app.world().get_entity(player).is_err());
        single::<DailyBill>(&mut app);
        assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, day);
        assert_eq!(app.world().resource::<Money>().0, balance);
        press_e(&mut app);
        KeyCode::KeyE.release(app.world_mut());
        // Paused weekends round to 30; 50 seconds commuting pays 25.
        balance += if weekend { 30 } else { 25 };
        if friday {
            balance -= 80;
        }
        advance(&mut app, 2);
        assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, day + 1);
        assert_eq!(app.world().resource::<RoadUnlocks>().0, unlocks);
        assert_absent::<DailyBill>(&mut app);
        assert_eq!(app.world().resource::<Money>().0, balance);
    }

    assert_eq!(weekend_roads.len(), 8);
    assert_eq!(weekend_roads[0], RoadId::MarketRoad);
    weekend_roads.sort_unstable();
    weekend_roads.dedup();
    assert_eq!(weekend_roads.len(), 8);
    assert_eq!(balance, 520);
    single::<InternshipEnded>(&mut app);
    app.world_mut().trigger(EndDay);
    advance(&mut app, 3);
    single::<DailyBill>(&mut app);
    assert_absent::<InternshipEnded>(&mut app);
    assert_eq!(app.world().resource::<Money>().0, balance);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 28);
}
