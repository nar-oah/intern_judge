use super::{
    InitialRoad, LevelPlugin, RoadEffect, RoadEffects, RoadId, SceneRoot, SwitchPoint,
    bill::{WeeklyBill, confirm_bill},
    day::{DayEndRequested, begin_next_day},
    navigation::{open_destinations, select_destination},
    scene::{
        effect::{ConstructionDetour, FallingObject},
        ending::InternshipEnded,
    },
    ui::{TravelButtonAction, TravelUiRoot, UiAction},
};
use crate::{
    config::{GameSettings, GameState},
    game::{
        CommuteState, CommuteTime, GamePlugin, HasHelmet, Money, Stamina, calendar::GameCalendar,
    },
    player::{Player, PlayerAction, PlayerPlugin, Stunned},
    test_support::{advance, finish_app, headless_app},
};
use avian2d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;
use bevy_replique::prelude::{DialogueFinished, DialogueRunner};
use leafwing_input_manager::prelude::{ActionState, Buttonlike, InputMap};

fn day_app(total_days: u32) -> App {
    let mut app = headless_app(Vec2::ZERO);
    app.world_mut().resource_mut::<GameSettings>().total_days = total_days;
    app.init_asset::<Font>()
        .init_asset_loader::<bevy::text::FontLoader>()
        .add_plugins((GamePlugin, PlayerPlugin, LevelPlugin));
    finish_app(&mut app);
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
fn next_day_resets_the_scene_and_daily_values_while_preserving_session_progress() {
    let mut app = day_app(28);
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
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 1);
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
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 1);
    assert_eq!(app.world().resource::<Money>().0, 41);
    assert!(app.world().resource::<HasHelmet>().0);
    assert_eq!(app.world().resource::<RoadEffects>().0, effects);
}

#[test]
fn requested_day_end_waits_for_dialogue_cleanup_and_runs_only_once() {
    let mut app = day_app(28);
    let (root, player) = assert_road(&mut app, RoadId::ExampleRoad);
    app.world_mut()
        .entity_mut(player)
        .insert(DialogueRunner::new(Handle::default()));
    app.world_mut().insert_resource(DayEndRequested);

    advance(&mut app, 2);

    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 0);
    assert_eq!(single::<SceneRoot>(&mut app), root);
    assert!(app.world().get::<DialogueRunner>(player).is_some());
    assert!(app.world().contains_resource::<DayEndRequested>());

    app.world_mut()
        .write_message(DialogueFinished { runner: player });
    app.update();

    assert!(app.world().get_entity(root).is_err());
    assert!(app.world().get_entity(player).is_err());
    assert_absent::<DialogueRunner>(&mut app);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 1);
    assert!(!app.world().contains_resource::<DayEndRequested>());
    let (new_root, _) = assert_road(&mut app, RoadId::ExampleRoad);
    advance(&mut app, 3);
    assert_eq!(single::<SceneRoot>(&mut app), new_root);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 1);
}

#[test]
fn friday_bill_waits_for_dialogue_cleanup_and_a_fresh_confirm_before_changing_money_or_date() {
    let mut app = day_app(28);
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

    // Repeated day-end requests cannot recreate the bill or settle it early.
    app.world_mut().insert_resource(DayEndRequested);
    app.update();
    assert_eq!(single::<WeeklyBill>(&mut app), bill);
    assert_eq!(app.world().resource::<Money>().0, 12);
    assert!(!app.world().contains_resource::<DayEndRequested>());
    KeyCode::KeyE.release(app.world_mut());
    app.update();
    KeyCode::KeyE.press(app.world_mut());
    app.update();

    assert_removed(&app, &bill_entities);
    assert_absent::<WeeklyBill>(&mut app);
    assert_road(&mut app, RoadId::ExampleRoad);
    assert_eq!(app.world().resource::<Money>().0, -4);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 7);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.0);
    assert_eq!(app.world().resource::<Stamina>().0, Stamina::MAX);
    assert!(app.world().resource::<HasHelmet>().0);
    confirm_bill(app.world_mut());
    advance(&mut app, 4);
    assert_eq!(app.world().resource::<Money>().0, -4);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 7);
    assert_commute(&app, CommuteState::Running);
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
    let mut app = day_app(28);
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
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 1);
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
            assert_road(&mut app, RoadId::ExampleRoad);
            assert_eq!(
                app.world().resource::<GameCalendar>().elapsed_days,
                elapsed_days
            );
            assert_eq!(
                app.world().resource::<GameCalendar>().remaining_days(),
                total_days - elapsed_days
            );
            assert_commute(&app, CommuteState::Running);
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
