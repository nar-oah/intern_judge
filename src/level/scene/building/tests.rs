use super::*;
use crate::config::{BuildingValues, ConvenienceKind, LotteryPrize};
use crate::game::{HasHelmet, Money, Stamina};
use crate::level::InteractRequested;
use crate::test_support::{finish_app, headless_app};

fn building_app() -> App {
    let mut app = headless_app(Vec2::ZERO);
    app.insert_resource(Money(100))
        .init_resource::<Stamina>()
        .init_resource::<HasHelmet>();
    finish_app(&mut app);
    app
}

fn interact(app: &mut App, entity: Entity) {
    app.world_mut().trigger(InteractRequested {
        entity,
        player: Entity::PLACEHOLDER,
    });
    app.world_mut().flush();
}

#[test]
fn convenience_store_charges_only_when_it_can_restore_stamina() {
    let mut app = building_app();
    let store = app
        .world_mut()
        .spawn_scene(convenience_store_scene(
            Vec2::ZERO,
            Vec2::splat(100.0),
            ConvenienceKind::Small,
        ))
        .unwrap()
        .id();

    for (money, stamina, expected_money, expected_stamina) in [
        (100, 40.0, 90, 70.0),
        (10, 90.0, 0, Stamina::MAX),
        (100, Stamina::MAX, 100, Stamina::MAX),
        (9, 40.0, 9, 40.0),
        (0, 0.0, 0, 0.0),
    ] {
        app.world_mut().resource_mut::<Money>().0 = money;
        app.world_mut().resource_mut::<Stamina>().0 = stamina;
        interact(&mut app, store);
        assert_eq!(app.world().resource::<Money>().0, expected_money);
        assert_eq!(app.world().resource::<Stamina>().0, expected_stamina);
        assert!(!app.world().resource::<HasHelmet>().0);
    }
}

#[test]
fn convenience_store_types_use_their_configured_price_and_recovery() {
    let mut app = building_app();
    {
        let mut values = app.world_mut().resource_mut::<BuildingValues>();
        let small = values
            .convenience_stores
            .get_mut(&ConvenienceKind::Small)
            .unwrap();
        small.price = 7;
        small.stamina_recovery = 17.0;
        let large = values
            .convenience_stores
            .get_mut(&ConvenienceKind::Large)
            .unwrap();
        large.price = 23;
        large.stamina_recovery = 61.0;
    }

    for (kind, price, recovery) in [
        (ConvenienceKind::Small, 7, 17.0),
        (ConvenienceKind::Large, 23, 61.0),
    ] {
        let store = app
            .world_mut()
            .spawn_scene(convenience_store_scene(
                Vec2::ZERO,
                Vec2::splat(100.0),
                kind,
            ))
            .unwrap()
            .id();
        assert_eq!(app.world().get::<ConvenienceKind>(store), Some(&kind));

        app.world_mut().resource_mut::<Money>().0 = 100;
        app.world_mut().resource_mut::<Stamina>().0 = 20.0;
        interact(&mut app, store);
        assert_eq!(app.world().resource::<Money>().0, 100 - price);
        assert_eq!(app.world().resource::<Stamina>().0, 20.0 + recovery);

        app.world_mut().resource_mut::<Money>().0 = price - 1;
        app.world_mut().resource_mut::<Stamina>().0 = 20.0;
        interact(&mut app, store);
        assert_eq!(app.world().resource::<Money>().0, price - 1);
        assert_eq!(app.world().resource::<Stamina>().0, 20.0);
    }
}

#[test]
fn tool_store_requires_twenty_yuan_and_does_not_sell_a_second_helmet() {
    let mut app = building_app();
    let store = app
        .world_mut()
        .spawn_scene(tool_store_scene(Vec2::ZERO, Vec2::splat(100.0)))
        .unwrap()
        .id();

    for (money, helmet, expected_money, expected_helmet) in [
        (100, false, 80, true),
        (20, false, 0, true),
        (19, false, 19, false),
        (100, true, 100, true),
    ] {
        app.world_mut().resource_mut::<Money>().0 = money;
        app.world_mut().resource_mut::<HasHelmet>().0 = helmet;
        interact(&mut app, store);
        assert_eq!(app.world().resource::<Money>().0, expected_money);
        assert_eq!(app.world().resource::<HasHelmet>().0, expected_helmet);
        assert_eq!(app.world().resource::<Stamina>().0, Stamina::MAX);
    }
}

#[test]
fn lottery_requires_a_ticket_payment_before_applying_a_prize() {
    let mut app = building_app();
    {
        let mut values = app.world_mut().resource_mut::<BuildingValues>();
        values.lottery.price = 13;
        values.lottery.prizes = vec![LotteryPrize {
            reward: 7,
            weight: 1,
        }];
    }
    let store = app
        .world_mut()
        .spawn_scene(lottery_store_scene(Vec2::ZERO, Vec2::splat(100.0)))
        .unwrap()
        .id();

    app.world_mut().resource_mut::<Money>().0 = 12;
    interact(&mut app, store);
    assert_eq!(app.world().resource::<Money>().0, 12);

    app.world_mut().resource_mut::<Money>().0 = 13;
    interact(&mut app, store);
    assert_eq!(app.world().resource::<Money>().0, 7);
}

#[test]
fn rescuing_a_cat_rewards_money_and_despawns_only_that_scene() {
    let mut app = building_app();
    app.world_mut().resource_mut::<BuildingValues>().cat_reward = 37;
    let cat = app
        .world_mut()
        .spawn_scene(cat_scene(Vec2::ZERO, Vec2::splat(30.0)))
        .unwrap()
        .id();
    let other_cat = app
        .world_mut()
        .spawn_scene(cat_scene(Vec2::X * 200.0, Vec2::splat(30.0)))
        .unwrap()
        .id();

    interact(&mut app, cat);
    assert_eq!(app.world().resource::<Money>().0, 137);
    assert!(app.world().get_entity(cat).is_err());
    assert!(app.world().get_entity(other_cat).is_ok());

    interact(&mut app, other_cat);
    assert_eq!(app.world().resource::<Money>().0, 174);
    assert!(app.world().get_entity(other_cat).is_err());
}
