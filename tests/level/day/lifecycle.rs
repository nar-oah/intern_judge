use super::support::assert_day_scene;
use crate::{
    config::{CaseDefs, GameState},
    game::{CommuteState, CommuteTime, DailyTransactions, HasHelmet, Money, Stamina, calendar::GameCalendar},
    level::{
        CurrentCase, InitialRoad, RoadEffect, RoadEffects, RoadId, SceneRoot, SelectedCase,
        SwitchPoint, TodayRoads,
        bill::{DailyBill, confirm_bill},
        day::{EndDay, start_day},
        scene::{
            effect::{ConstructionDetour, FallingObject},
            ending::InternshipEnded,
        },
        tests::support::{assert_commute, assert_road, assert_today_roads, day_app, weekday_app},
        ui::TravelUiRoot,
    },
    player::{Player, PlayerAction, Stunned},
    test_support::{advance, assert_absent, assert_removed, named, press_e, scene_entities, single},
};
use avian2d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;
use bevy_replique::prelude::DialogueRunner;
use leafwing_input_manager::prelude::ActionState;

#[test]
fn starting_any_day_resets_the_previous_roads_and_case_before_choosing_its_scene() {
    for elapsed_days in [0, 2, 28] {
        let mut app = day_app(28);
        app.world_mut().resource_mut::<GameCalendar>().elapsed_days = elapsed_days;
        app.world_mut().resource_mut::<TodayRoads>().0 =
            [RoadId::MarketRoad, RoadId::EmptyRoad1].into();
        let case = app.world().resource::<CaseDefs>().0[0].clone();
        app.world_mut().resource_mut::<CurrentCase>().0 = Some(SelectedCase {
            road: RoadId::MarketRoad,
            case,
        });

        app.world_mut().run_system_cached(start_day).unwrap();

        assert!(app.world().resource::<CurrentCase>().0.is_none());
        if elapsed_days == 2 {
            assert_road(&mut app, RoadId::ExampleRoad);
            assert_today_roads(&app, &[RoadId::ExampleRoad]);
        } else {
            assert_today_roads(&app, &[]);
            assert_absent::<Player>(&mut app);
            if elapsed_days == 28 {
                single::<InternshipEnded>(&mut app);
            } else {
                single::<TravelUiRoot>(&mut app);
            }
        }
    }
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

    app.world_mut().resource_mut::<DailyTransactions>().cat_reward = 7;
    app.world_mut().trigger(EndDay);
    app.update();
    press_e(&mut app);

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
    assert_eq!(app.world().resource::<Money>().0, 48);
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
    assert_eq!(app.world().resource::<Money>().0, 48);
    assert!(app.world().resource::<HasHelmet>().0);
    assert_eq!(app.world().resource::<RoadEffects>().0, effects);
}

#[test]
fn end_day_immediately_replaces_the_scene_and_repeated_events_keep_the_same_bill() {
    let mut app = weekday_app(28);
    let (root, player) = assert_road(&mut app, RoadId::ExampleRoad);
    app.world_mut()
        .entity_mut(player)
        .insert(DialogueRunner::new(Handle::default()));

    app.world_mut().trigger(EndDay);

    assert!(app.world().get_entity(root).is_err());
    assert!(app.world().get_entity(player).is_err());
    assert_absent::<DialogueRunner>(&mut app);
    assert_absent::<Player>(&mut app);
    let bill = single::<DailyBill>(&mut app);
    assert_eq!(single::<SceneRoot>(&mut app), bill);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 2);

    app.world_mut().trigger(EndDay);
    advance(&mut app, 3);

    assert_eq!(single::<SceneRoot>(&mut app), bill);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 2);

    press_e(&mut app);
    advance(&mut app, 3);

    assert_absent::<DailyBill>(&mut app);
    assert_road(&mut app, RoadId::ExampleRoad);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 3);
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
            let huds = ["CommuteHudRoot", "CalendarHudRoot", "CaseHudRoot"].map(|name| {
                let root = named(&mut app, name);
                assert!(app.world().get::<SceneRoot>(root).is_none());
                assert!(app.world().get::<ChildOf>(root).is_none());
                (root, scene_entities(app.world(), root))
            });
            let mut previous = Vec::new();
            for _ in 0..root_count {
                let root = app.world_mut().spawn(SceneRoot).id();
                let child = app.world_mut().spawn(ChildOf(root)).id();
                previous.extend([root, child]);
            }

            app.world_mut().trigger(EndDay);
            if !billing {
                app.update();
                press_e(&mut app);
            }

            assert_removed(&app, &previous);
            let root = single::<SceneRoot>(&mut app);
            if billing {
                assert_eq!(single::<DailyBill>(&mut app), root);
                assert_absent::<Player>(&mut app);
                assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 2);
            } else {
                assert_road(&mut app, RoadId::ExampleRoad);
                assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 3);
            }
            assert!(app.world().get_entity(camera).is_ok());
            assert!(app.world().get_entity(unrelated).is_ok());
            for (hud, entities) in huds {
                assert_eq!(scene_entities(app.world(), hud), entities);
                for entity in entities {
                    assert!(app.world().get_entity(entity).is_ok());
                }
            }
        }
    }
}

#[test]
fn ending_the_last_day_waits_for_its_bill_and_cannot_restart_the_commute() {
    for total_days in [1, 3, 28] {
        let mut app = day_app(total_days);
        app.world_mut().resource_mut::<Money>().0 = 59;
        app.world_mut().resource_mut::<HasHelmet>().0 = true;
        let mut balance = 59;
        for elapsed_days in 1..total_days {
            let root = single::<SceneRoot>(&mut app);
            let calendar = app.world().resource::<GameCalendar>();
            if calendar.weekday() == 5 {
                let settings = app.world().resource::<crate::config::GameSettings>();
                balance += settings.weekly_salary - settings.weekly_rent;
            }
            app.world_mut().trigger(EndDay);
            app.update();
            single::<DailyBill>(&mut app);
            press_e(&mut app);
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
            assert_eq!(app.world().resource::<Money>().0, balance);
        }
        let old_root = single::<SceneRoot>(&mut app);
        let old_scene = scene_entities(app.world(), old_root);
        app.world_mut().resource_mut::<CommuteTime>().elapsed = 17.0;
        app.world_mut().resource_mut::<Stamina>().0 = 2.0;
        app.world_mut().resource_mut::<DailyTransactions>().cat_reward = 3;
        if app.world().resource::<GameCalendar>().weekday() == 5 {
            let settings = app.world().resource::<crate::config::GameSettings>();
            balance += settings.weekly_salary - settings.weekly_rent;
        }

        app.world_mut().trigger(EndDay);
        advance(&mut app, 3);

        assert_removed(&app, &old_scene);
        single::<DailyBill>(&mut app);
        assert_absent::<InternshipEnded>(&mut app);
        assert_eq!(
            app.world().resource::<GameCalendar>().elapsed_days,
            total_days - 1
        );
        assert_eq!(app.world().resource::<DailyTransactions>().cat_reward, 3);
        press_e(&mut app);
        app.update();

        let root = single::<SceneRoot>(&mut app);
        assert_eq!(single::<InternshipEnded>(&mut app), root);
        assert_absent::<DailyBill>(&mut app);
        assert_absent::<Player>(&mut app);
        assert_absent::<RoadId>(&mut app);
        assert_absent::<SwitchPoint>(&mut app);
        let calendar = app.world().resource::<GameCalendar>();
        assert_eq!(calendar.elapsed_days, total_days);
        assert_eq!(calendar.remaining_days(), 0);
        assert!(calendar.is_finished());
        assert_eq!(app.world().resource::<Stamina>().0, Stamina::MAX);
        assert_eq!(app.world().resource::<Money>().0, balance + 3);
        assert_eq!(
            *app.world().resource::<DailyTransactions>(),
            DailyTransactions::default()
        );
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

        app.world_mut().run_system_cached(confirm_bill).unwrap();
        app.world_mut().trigger(EndDay);
        advance(&mut app, 4);

        assert_eq!(single::<SceneRoot>(&mut app), root);
        assert_absent::<DailyBill>(&mut app);
        assert_absent::<Player>(&mut app);
        assert_absent::<RoadId>(&mut app);
        assert_eq!(
            app.world().resource::<GameCalendar>().elapsed_days,
            total_days
        );
        assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.0);
        assert_eq!(app.world().resource::<Money>().0, balance + 3);
        assert_commute(&app, CommuteState::Paused);
    }
}
