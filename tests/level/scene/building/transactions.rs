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
        app.world_mut().resource_mut::<Stamina>().0 = stamina;

        interact(&mut app, store);

        assert_eq!(app.world().resource::<Money>().0, 0, "{kind:?}");
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
    assert!(!app.world().resource::<HasHelmet>().0);

    app.world_mut().resource_mut::<Money>().0 = 20;
    interact(&mut app, store);
    assert_eq!(app.world().resource::<Money>().0, 0);
    assert!(app.world().resource::<HasHelmet>().0);

    app.world_mut().resource_mut::<Money>().0 = 100;
    interact(&mut app, store);
    assert_eq!(app.world().resource::<Money>().0, 100);
    assert!(app.world().resource::<HasHelmet>().0);
}

#[test]
fn lottery_requires_payment_before_awarding_the_prize() {
    let mut app = building_app();
    let store = spawn_building(&mut app, "LotteryStore");

    app.world_mut().resource_mut::<Money>().0 = 6;
    interact(&mut app, store);
    assert_eq!(app.world().resource::<Money>().0, 6);

    app.world_mut().resource_mut::<Money>().0 = 7;
    interact(&mut app, store);
    assert_eq!(app.world().resource::<Money>().0, 13);
}

#[test]
fn rescuing_a_cat_pays_the_reward_and_removes_only_that_cat() {
    let mut app = building_app();
    let cat = spawn_building(&mut app, "Cat");
    let other_cat = spawn_building(&mut app, "Cat");

    interact(&mut app, cat);

    assert_eq!(app.world().resource::<Money>().0, 117);
    assert_removed(&app, &[cat]);
    assert!(app.world().get_entity(other_cat).is_ok());
}
