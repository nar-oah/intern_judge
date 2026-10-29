use super::*;
use crate::config::{
    BuildingValues, ConvenienceKind, ConvenienceValues, LotteryPrize, LotteryValues,
};
use crate::game::{HasHelmet, Money, Stamina};
use crate::level::InteractRequested;
use bevy::{app::TaskPoolPlugin, asset::AssetPlugin, scene::ScenePlugin};

fn building_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        TaskPoolPlugin::default(),
        AssetPlugin::default(),
        ScenePlugin,
    ))
    .insert_resource(BuildingValues {
        cat_reward: 37,
        lottery: LotteryValues {
            price: 13,
            prizes: vec![LotteryPrize {
                reward: 7,
                weight: 1,
            }],
        },
        convenience_stores: [
            (
                ConvenienceKind::Small,
                ConvenienceValues {
                    price: 7,
                    stamina_recovery: 17.0,
                },
            ),
            (
                ConvenienceKind::Large,
                ConvenienceValues {
                    price: 23,
                    stamina_recovery: 61.0,
                },
            ),
        ]
        .into(),
    })
    .insert_resource(Money(100))
    .init_resource::<Stamina>()
    .init_resource::<HasHelmet>();
    app.finish();
    app.cleanup();
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
fn convenience_stores_charge_the_selected_offer_and_cap_recovery() {
    let mut app = building_app();
    for (kind, price, stamina, expected_stamina) in [
        (ConvenienceKind::Small, 7, 40.0, 57.0),
        (ConvenienceKind::Large, 23, 60.0, Stamina::MAX),
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
    let store = app
        .world_mut()
        .spawn_scene(convenience_store_scene(
            Vec2::ZERO,
            Vec2::ONE,
            ConvenienceKind::Small,
        ))
        .unwrap()
        .id();

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
    let store = app
        .world_mut()
        .spawn_scene(tool_store_scene(Vec2::ZERO, Vec2::ONE))
        .unwrap()
        .id();

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
    let store = app
        .world_mut()
        .spawn_scene(lottery_store_scene(Vec2::ZERO, Vec2::ONE))
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
fn rescuing_a_cat_pays_the_reward_and_removes_only_that_cat() {
    let mut app = building_app();
    let cat = app
        .world_mut()
        .spawn_scene(cat_scene(Vec2::ZERO, Vec2::ONE))
        .unwrap()
        .id();
    let other_cat = app
        .world_mut()
        .spawn_scene(cat_scene(Vec2::X, Vec2::ONE))
        .unwrap()
        .id();

    interact(&mut app, cat);

    assert_eq!(app.world().resource::<Money>().0, 137);
    assert!(app.world().get_entity(cat).is_err());
    assert!(app.world().get_entity(other_cat).is_ok());
}
