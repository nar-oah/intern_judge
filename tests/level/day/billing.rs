use super::support::calendar_hud;
use crate::{
    config::GameSettings,
    game::{CommuteState, CommuteTime, HasHelmet, Money, Stamina, calendar::GameCalendar},
    level::{
        RoadId, SceneRoot,
        bill::{WeeklyBill, confirm_bill},
        day::DayEndRequested,
        tests::support::{assert_commute, assert_road, weekday_app},
        ui::{TravelUiRoot, UiAction},
    },
    player::Player,
    test_support::{advance, assert_absent, assert_removed, press_e, scene_entities, single},
};
use bevy::prelude::*;
use bevy_replique::prelude::{DialogueFinished, DialogueRunner};
use leafwing_input_manager::prelude::{ActionState, Buttonlike, InputMap};

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
    press_e(&mut app);

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
