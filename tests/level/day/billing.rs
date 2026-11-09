use super::support::calendar_hud;
use crate::{
    config::GameSettings,
    game::{
        CommuteState, CommuteTime, DailyTransactions, HasHelmet, Money, Stamina,
        calendar::GameCalendar,
    },
    level::{
        RoadId, SceneRoot,
        bill::{DailyBill, confirm_bill},
        day::EndDay,
        tests::support::{assert_commute, assert_road, weekday_app},
        ui::{TravelUiRoot, UiAction},
    },
    player::Player,
    test_support::{advance, assert_absent, assert_removed, press_e, scene_entities, single},
};
use bevy::prelude::*;
use leafwing_input_manager::prelude::{ActionState, Buttonlike, InputMap};

#[test]
fn friday_bill_snapshots_all_income_and_expenses_and_waits_for_a_fresh_confirm() {
    let mut app = weekday_app(28);
    let (root, _) = assert_road(&mut app, RoadId::ExampleRoad);
    let old_scene = scene_entities(app.world(), root);
    let (hud, text, _) = calendar_hud(&mut app);
    app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 6;
    app.world_mut().resource_mut::<Money>().0 = 12;
    *app.world_mut().resource_mut::<DailyTransactions>() = DailyTransactions {
        convenience_store_cost: 3,
        tool_store_cost: 4,
        cat_reward: 5,
        lottery_cost: 6,
        lottery_reward: 7,
    };
    let transactions = *app.world().resource::<DailyTransactions>();
    app.world_mut().resource_mut::<HasHelmet>().0 = true;
    app.world_mut().resource_mut::<CommuteTime>().elapsed = 4.0;
    app.world_mut().resource_mut::<Stamina>().0 = 4.0;
    KeyCode::KeyE.press(app.world_mut());
    app.world_mut().trigger(EndDay);
    advance(&mut app, 4);

    assert_removed(&app, &old_scene);
    assert_absent::<Player>(&mut app);
    let bill = single::<DailyBill>(&mut app);
    assert_eq!(single::<SceneRoot>(&mut app), bill);
    assert!(app.world().get::<InputMap<UiAction>>(bill).is_some());
    assert!(app.world().get::<ActionState<UiAction>>(bill).is_some());
    assert_commute(&app, CommuteState::Paused);
    assert_eq!(app.world().resource::<Money>().0, 12);
    assert_eq!(*app.world().resource::<DailyTransactions>(), transactions);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 6);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 4.0);
    assert_eq!(app.world().resource::<Stamina>().0, 4.0);
    let (new_hud, new_text, date) = calendar_hud(&mut app);
    assert_eq!((new_hud, new_text), (hud, text));
    assert!(date.contains("Friday"));
    let bill_entities = scene_entities(app.world(), bill);
    let summary = bill_entities
        .iter()
        .find_map(|&entity| app.world().get::<Text>(entity))
        .unwrap()
        .0
        .clone();
    for expected in [
        "每日账单",
        "当前余额：12",
        "工资：31",
        "房租：47",
        "便利店付款：3",
        "工具店付款：4",
        "救助猫咪收益：5",
        "彩票店付款：6",
        "彩票中奖收益：7",
        "结算后余额：-5",
    ] {
        assert!(
            summary.lines().any(|line| line == expected),
            "missing {expected}: {summary}"
        );
    }

    {
        let mut settings = app.world_mut().resource_mut::<GameSettings>();
        settings.base_salary = 103;
        settings.work_time = 0.5;
        settings.commute_limit = 100.0;
        settings.weekly_rent = 7;
    }
    app.world_mut().resource_mut::<CommuteTime>().elapsed = 0.0;
    app.world_mut().resource_mut::<Money>().0 = 500;
    app.world_mut()
        .resource_mut::<DailyTransactions>()
        .lottery_reward = 99;
    press_e(&mut app);

    assert_removed(&app, &bill_entities);
    assert_absent::<DailyBill>(&mut app);
    single::<TravelUiRoot>(&mut app);
    assert_absent::<Player>(&mut app);
    assert_eq!(app.world().resource::<Money>().0, -5);
    assert_eq!(
        *app.world().resource::<DailyTransactions>(),
        DailyTransactions::default()
    );
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 7);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.0);
    assert_eq!(app.world().resource::<Stamina>().0, Stamina::MAX);
    assert!(app.world().resource::<HasHelmet>().0);
    app.world_mut().run_system_cached(confirm_bill).unwrap();
    advance(&mut app, 4);
    assert_eq!(app.world().resource::<Money>().0, -5);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 7);
    assert_commute(&app, CommuteState::Paused);
}

#[test]
fn bills_pay_daily_salary_without_transactions_and_only_friday_charges_rent() {
    for elapsed_days in (0..7).chain([28, 29, 34, 35]) {
        let mut app = weekday_app(28);
        app.world_mut().resource_mut::<GameCalendar>().elapsed_days = elapsed_days;
        app.world_mut().resource_mut::<Money>().0 = 19;
        app.world_mut().resource_mut::<CommuteTime>().elapsed = 4.0;
        let friday = app.world().resource::<GameCalendar>().weekday() == 5;
        let settings = app.world().resource::<GameSettings>();
        let salary = 31;
        let rent = if friday { settings.weekly_rent } else { 0 };

        app.world_mut().trigger(EndDay);
        advance(&mut app, 3);

        let bill = single::<DailyBill>(&mut app);
        let summary = scene_entities(app.world(), bill)
            .iter()
            .find_map(|&entity| app.world().get::<Text>(entity))
            .unwrap()
            .0
            .clone();
        assert!(summary.lines().any(|line| line == format!("工资：{salary}")));
        assert!(summary.contains(&format!("房租：{rent}")));
        for label in [
            "便利店付款",
            "工具店付款",
            "救助猫咪收益",
            "彩票店付款",
            "彩票中奖收益",
        ] {
            assert!(summary.contains(&format!("{label}：0")));
        }
        assert_eq!(app.world().resource::<Money>().0, 19);
        assert_eq!(
            app.world().resource::<GameCalendar>().elapsed_days,
            elapsed_days
        );

        press_e(&mut app);

        assert_absent::<DailyBill>(&mut app);
        assert_eq!(app.world().resource::<Money>().0, 19 + salary - rent);
        assert_eq!(
            app.world().resource::<GameCalendar>().elapsed_days,
            elapsed_days + 1
        );
    }
}

#[test]
fn daily_salary_scales_with_remaining_commute_time_and_rounds_positive_and_negative_values() {
    let mut app = weekday_app(28);
    {
        let mut settings = app.world_mut().resource_mut::<GameSettings>();
        settings.base_salary = 10;
        settings.work_time = 2.5;
        settings.commute_limit = 10.0;
    }
    for (elapsed, salary) in [
        (0.0, 40),
        (5.0, 20),
        (7.5, 10),
        (8.625, 6),
        (8.875, 5),
        (8.9375, 4),
        (10.0, 0),
        (10.3125, -1),
        (10.375, -2),
        (12.5, -10),
    ] {
        app.world_mut().resource_mut::<CommuteTime>().elapsed = elapsed;
        app.world_mut().resource_mut::<Money>().0 = 19;

        app.world_mut().trigger(EndDay);
        advance(&mut app, 3);

        let bill = single::<DailyBill>(&mut app);
        let expected = format!("工资：{salary}");
        assert!(
            scene_entities(app.world(), bill).iter().any(|&entity| {
                app.world()
                    .get::<Text>(entity)
                    .is_some_and(|text| text.0.lines().any(|line| line == expected))
            }),
            "elapsed {elapsed} should produce salary {salary}"
        );
        assert_eq!(app.world().resource::<Money>().0, 19);

        press_e(&mut app);

        assert_absent::<DailyBill>(&mut app);
        assert_eq!(
            app.world().resource::<Money>().0,
            19 + salary,
            "elapsed {elapsed} should settle salary {salary}"
        );
        // Keep each case on Monday so the table isolates salary from Friday rent.
        app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 2;
    }
}
