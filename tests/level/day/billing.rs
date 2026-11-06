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
    app.world_mut().resource_mut::<CommuteTime>().elapsed = 17.0;
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
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 17.0);
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
        "本周工资：31",
        "房租：47",
        "便利店付款：3",
        "工具店付款：4",
        "救助猫咪收益：5",
        "彩票店付款：6",
        "彩票中奖收益：7",
        "结算后余额：-5",
    ] {
        assert!(summary.contains(expected), "missing {expected}: {summary}");
    }

    {
        let mut settings = app.world_mut().resource_mut::<GameSettings>();
        settings.weekly_salary = 103;
        settings.weekly_rent = 7;
    }
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
fn bills_show_before_and_after_the_last_day_without_transactions_and_only_friday_charges_weekly_items() {
    for elapsed_days in (0..7).chain([28, 29, 34, 35]) {
        let mut app = weekday_app(28);
        app.world_mut().resource_mut::<GameCalendar>().elapsed_days = elapsed_days;
        app.world_mut().resource_mut::<Money>().0 = 19;
        let friday = app.world().resource::<GameCalendar>().weekday() == 5;
        let settings = app.world().resource::<GameSettings>();
        let (salary, rent) = if friday {
            (settings.weekly_salary, settings.weekly_rent)
        } else {
            (0, 0)
        };

        app.world_mut().trigger(EndDay);
        advance(&mut app, 3);

        let bill = single::<DailyBill>(&mut app);
        let summary = scene_entities(app.world(), bill)
            .iter()
            .find_map(|&entity| app.world().get::<Text>(entity))
            .unwrap()
            .0
            .clone();
        assert!(summary.contains(&format!("本周工资：{salary}")));
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
