use super::*;
use crate::test_support::assert_removed;

#[test]
fn convenience_stores_charge_the_selected_offer_and_cap_recovery() {
    let mut app = building_app();
    for (kind, price, stamina, expected_stamina) in [
        (ConvenienceKind::Small, 7, 40.0, 65.0),
        (ConvenienceKind::Large, 13, 60.0, Stamina::MAX),
    ] {
        let store = app
            .world_mut()
            .spawn_scene(convenience_store_scene(Vec2::ZERO, Vec2::ONE, kind))
            .unwrap()
            .id();
        app.world_mut().resource_mut::<Money>().0 = price;
        *app.world_mut().resource_mut::<DailyTransactions>() = DailyTransactions::default();
        app.world_mut().resource_mut::<Stamina>().0 = stamina;

        interact(&mut app, store);

        assert_eq!(app.world().resource::<Money>().0, price, "{kind:?}");
        assert_eq!(
            *app.world().resource::<DailyTransactions>(),
            DailyTransactions {
                convenience_store_cost: price,
                ..default()
            },
            "{kind:?}"
        );
        assert_eq!(
            app.world().resource::<Stamina>().0,
            expected_stamina,
            "{kind:?}"
        );
    }
}

#[test]
fn convenience_store_rejects_full_stamina_or_insufficient_money() {
    let mut app = building_app();
    let store = spawn_building(&mut app, "ConvenienceStore");

    for (money, stamina) in [(100, Stamina::MAX), (6, 40.0)] {
        app.world_mut().resource_mut::<Money>().0 = money;
        app.world_mut().resource_mut::<Stamina>().0 = stamina;

        interact(&mut app, store);

        assert_eq!(app.world().resource::<Money>().0, money);
        assert_eq!(
            *app.world().resource::<DailyTransactions>(),
            DailyTransactions::default()
        );
        assert_eq!(app.world().resource::<Stamina>().0, stamina);
    }
}

#[test]
fn tool_store_charges_only_for_the_first_affordable_helmet() {
    let mut app = building_app();
    let store = spawn_building(&mut app, "ToolStore");

    app.world_mut().resource_mut::<Money>().0 = 19;
    interact(&mut app, store);
    assert_eq!(app.world().resource::<Money>().0, 19);
    assert_eq!(
        *app.world().resource::<DailyTransactions>(),
        DailyTransactions::default()
    );
    assert!(!app.world().resource::<HasHelmet>().0);

    app.world_mut().resource_mut::<Money>().0 = 20;
    interact(&mut app, store);
    assert_eq!(app.world().resource::<Money>().0, 20);
    assert_eq!(app.world().resource::<DailyTransactions>().tool_store_cost, 20);
    assert!(app.world().resource::<HasHelmet>().0);

    app.world_mut().resource_mut::<Money>().0 = 100;
    interact(&mut app, store);
    assert_eq!(app.world().resource::<Money>().0, 100);
    assert_eq!(app.world().resource::<DailyTransactions>().tool_store_cost, 20);
    assert!(app.world().resource::<HasHelmet>().0);
}

#[test]
fn lottery_requires_payment_before_awarding_the_prize() {
    let mut app = building_app();
    let store = spawn_building(&mut app, "LotteryStore");

    app.world_mut().resource_mut::<Money>().0 = 6;
    interact(&mut app, store);
    assert_eq!(app.world().resource::<Money>().0, 6);
    assert_eq!(
        *app.world().resource::<DailyTransactions>(),
        DailyTransactions::default()
    );

    app.world_mut().resource_mut::<Money>().0 = 7;
    interact(&mut app, store);
    assert_eq!(app.world().resource::<Money>().0, 7);
    assert_eq!(
        *app.world().resource::<DailyTransactions>(),
        DailyTransactions {
            lottery_cost: 7,
            lottery_reward: 13,
            ..default()
        }
    );
}

#[test]
fn rescuing_a_cat_pays_the_reward_and_removes_only_that_cat() {
    let mut app = building_app();
    let cat = spawn_building(&mut app, "Cat");
    let other_cat = spawn_building(&mut app, "Cat");

    interact(&mut app, cat);

    assert_eq!(app.world().resource::<Money>().0, 100);
    assert_eq!(app.world().resource::<DailyTransactions>().cat_reward, 17);
    assert_removed(&app, &[cat]);
    assert!(app.world().get_entity(other_cat).is_ok());

    interact(&mut app, other_cat);

    assert_eq!(app.world().resource::<Money>().0, 100);
    assert_eq!(app.world().resource::<DailyTransactions>().cat_reward, 34);
    assert_removed(&app, &[other_cat]);
}

#[test]
fn repeated_purchases_cannot_spend_the_same_money_twice() {
    for building in ["ConvenienceStore", "LotteryStore"] {
        let mut app = building_app();
        app.world_mut().resource_mut::<Money>().0 = 14;
        app.world_mut().resource_mut::<Stamina>().0 = 0.0;
        app.world_mut()
            .resource_mut::<crate::config::BuildingValues>()
            .lottery
            .prizes[0]
            .reward = 0;
        let store = spawn_building(&mut app, building);

        for _ in 0..3 {
            interact(&mut app, store);
        }

        assert_eq!(app.world().resource::<Money>().0, 14, "{building}");
        let transactions = app.world().resource::<DailyTransactions>();
        assert_eq!(transactions.net(), -14, "{building}");
        if building == "ConvenienceStore" {
            assert_eq!(transactions.convenience_store_cost, 14);
            assert_eq!(app.world().resource::<Stamina>().0, 50.0);
        } else {
            assert_eq!(transactions.lottery_cost, 14);
        }
    }
}

#[test]
fn helmet_payment_reduces_the_balance_available_at_other_stores() {
    let mut app = building_app();
    app.world_mut().resource_mut::<Money>().0 = 26;
    app.world_mut().resource_mut::<Stamina>().0 = 35.0;
    let tools = spawn_building(&mut app, "ToolStore");
    let convenience = spawn_building(&mut app, "ConvenienceStore");
    let lottery = spawn_building(&mut app, "LotteryStore");

    interact(&mut app, tools);
    interact(&mut app, convenience);
    interact(&mut app, lottery);

    assert_eq!(app.world().resource::<Money>().0, 26);
    assert_eq!(
        *app.world().resource::<DailyTransactions>(),
        DailyTransactions {
            tool_store_cost: 20,
            ..default()
        }
    );
    assert!(app.world().resource::<HasHelmet>().0);
    assert_eq!(app.world().resource::<Stamina>().0, 35.0);
}

#[test]
fn pending_cat_rewards_can_be_spent_at_every_store() {
    for (building, expected) in [
        (
            "ConvenienceStore",
            DailyTransactions {
                convenience_store_cost: 7,
                cat_reward: 17,
                ..default()
            },
        ),
        (
            "ToolStore",
            DailyTransactions {
                tool_store_cost: 20,
                cat_reward: 17,
                ..default()
            },
        ),
        (
            "LotteryStore",
            DailyTransactions {
                cat_reward: 17,
                lottery_cost: 7,
                lottery_reward: 13,
                ..default()
            },
        ),
    ] {
        let mut app = building_app();
        app.world_mut().resource_mut::<Money>().0 = 3;
        app.world_mut().resource_mut::<Stamina>().0 = 35.0;
        let cat = spawn_building(&mut app, "Cat");
        let store = spawn_building(&mut app, building);

        interact(&mut app, cat);
        interact(&mut app, store);

        assert_eq!(app.world().resource::<Money>().0, 3, "{building}");
        assert_eq!(
            *app.world().resource::<DailyTransactions>(),
            expected,
            "{building}"
        );
    }
}

#[test]
fn pending_lottery_prizes_can_fund_a_helmet() {
    let mut app = building_app();
    app.world_mut().resource_mut::<Money>().0 = 20;
    let lottery = spawn_building(&mut app, "LotteryStore");
    let tools = spawn_building(&mut app, "ToolStore");

    interact(&mut app, lottery);
    interact(&mut app, tools);

    assert_eq!(app.world().resource::<Money>().0, 20);
    assert_eq!(
        *app.world().resource::<DailyTransactions>(),
        DailyTransactions {
            tool_store_cost: 20,
            lottery_cost: 7,
            lottery_reward: 13,
            ..default()
        }
    );
    assert!(app.world().resource::<HasHelmet>().0);
}
