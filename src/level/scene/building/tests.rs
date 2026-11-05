use super::*;
use crate::config::{
    BuildingValues, ConvenienceKind, ConvenienceValues, LotteryPrize, LotteryValues,
};
use crate::game::{CommuteState, HasHelmet, Money, Stamina, calendar::GameCalendar};
use crate::level::{BuildingType, InteractRequested};
use bevy::{app::TaskPoolPlugin, asset::AssetPlugin, scene::ScenePlugin};
use bevy_replique::prelude::{
    DialogueChoices, DialogueCommand, DialogueFinished, DialogueLine, RepliquePlugin,
    ResumeDialogue, ResumeInput,
};
use std::time::{Duration, Instant};

fn building_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        TaskPoolPlugin::default(),
        AssetPlugin::default(),
        ScenePlugin,
        RepliquePlugin,
    ))
    .init_asset::<Font>()
    .insert_resource(State::new(CommuteState::Running))
    .insert_resource(GameCalendar {
        total_days: 28,
        elapsed_days: 2,
    })
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

fn spawn_building(app: &mut App, building: BuildingType) -> Entity {
    match building {
        BuildingType::ConvenienceStore => app
            .world_mut()
            .spawn_scene(convenience_store_scene(
                Vec2::ZERO,
                Vec2::ONE,
                ConvenienceKind::Small,
            ))
            .unwrap()
            .id(),
        BuildingType::ToolStore => app
            .world_mut()
            .spawn_scene(tool_store_scene(Vec2::ZERO, Vec2::ONE))
            .unwrap()
            .id(),
        BuildingType::LotteryStore => app
            .world_mut()
            .spawn_scene(lottery_store_scene(Vec2::ZERO, Vec2::ONE))
            .unwrap()
            .id(),
        BuildingType::Cat => app
            .world_mut()
            .spawn_scene(cat_scene(Vec2::ZERO, Vec2::ONE))
            .unwrap()
            .id(),
    }
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

#[test]
fn paused_buildings_start_dialogue_without_running_their_effects() {
    for building in [
        BuildingType::ConvenienceStore,
        BuildingType::ToolStore,
        BuildingType::LotteryStore,
        BuildingType::Cat,
    ] {
        let mut app = building_app();
        app.insert_resource(State::new(CommuteState::Paused));
        app.world_mut().resource_mut::<Stamina>().0 = 35.0;
        let entity = spawn_building(&mut app, building);
        let player = app
            .world_mut()
            .spawn((Player::default(), LinearVelocity(Vec2::new(42.0, 7.0))))
            .id();

        app.world_mut()
            .trigger(InteractRequested { entity, player });
        app.world_mut().flush();

        assert!(app.world().get::<DialogueRunner>(player).is_some());
        assert_eq!(
            app.world().get::<LinearVelocity>(player).unwrap().0,
            Vec2::new(0.0, 7.0)
        );
        assert_eq!(app.world().resource::<Money>().0, 100, "{building:?}");
        assert_eq!(app.world().resource::<Stamina>().0, 35.0, "{building:?}");
        assert!(!app.world().resource::<HasHelmet>().0, "{building:?}");
        assert!(app.world().get_entity(entity).is_ok(), "{building:?}");
        let requests = app
            .world_mut()
            .resource_mut::<Messages<StartDialogue>>()
            .drain()
            .collect::<Vec<_>>();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].runner, player);
        assert_eq!(
            requests[0].node.as_str(),
            match building {
                BuildingType::ConvenienceStore => "convenience_store",
                BuildingType::ToolStore => "tool_store",
                BuildingType::LotteryStore => "lottery_store",
                BuildingType::Cat => "cat",
            }
        );
    }
}

#[test]
fn every_building_dialogue_option_is_information_only() {
    for building in [
        BuildingType::ConvenienceStore,
        BuildingType::ToolStore,
        BuildingType::LotteryStore,
        BuildingType::Cat,
    ] {
        for choice in 0..2 {
            let mut app = building_app();
            app.insert_resource(State::new(CommuteState::Paused));
            app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 0;
            app.world_mut().resource_mut::<Stamina>().0 = 35.0;
            let entity = spawn_building(&mut app, building);
            let player = app
                .world_mut()
                .spawn((Player::default(), LinearVelocity::ZERO))
                .id();
            app.world_mut()
                .trigger(InteractRequested { entity, player });
            app.world_mut().flush();
            let deadline = Instant::now() + Duration::from_secs(10);
            let mut selected = false;
            let mut line_count = 0;

            loop {
                app.update();
                let lines = app
                    .world_mut()
                    .resource_mut::<Messages<DialogueLine>>()
                    .drain()
                    .collect::<Vec<_>>();
                for line in lines {
                    assert!(!line.text.is_empty());
                    line_count += 1;
                    app.world_mut().write_message(ResumeDialogue {
                        token: line.token,
                        input: ResumeInput::Advance,
                    });
                }
                let choices = app
                    .world_mut()
                    .resource_mut::<Messages<DialogueChoices>>()
                    .drain()
                    .collect::<Vec<_>>();
                for options in choices {
                    assert_eq!(options.choices.len(), 2);
                    assert!(options.choices.iter().all(|choice| choice.available));
                    assert!(!selected);
                    selected = true;
                    app.world_mut().write_message(ResumeDialogue {
                        token: options.token,
                        input: ResumeInput::Select(choice),
                    });
                }
                assert!(
                    app.world_mut()
                        .resource_mut::<Messages<DialogueCommand>>()
                        .drain()
                        .next()
                        .is_none(),
                    "{building:?}, choice {choice} must not run commands"
                );
                if app
                    .world_mut()
                    .resource_mut::<Messages<DialogueFinished>>()
                    .drain()
                    .any(|finished| finished.runner == player)
                {
                    break;
                }
                assert!(
                    Instant::now() < deadline,
                    "{building:?}, choice {choice} dialogue did not finish"
                );
                std::thread::sleep(Duration::from_millis(1));
            }

            assert!(selected);
            assert_eq!(line_count, if choice == 0 { 2 } else { 1 });
            assert_eq!(app.world().resource::<Money>().0, 100);
            assert_eq!(app.world().resource::<Stamina>().0, 35.0);
            assert!(!app.world().resource::<HasHelmet>().0);
            assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 0);
            assert!(app.world().get_entity(entity).is_ok());
        }
    }
}
