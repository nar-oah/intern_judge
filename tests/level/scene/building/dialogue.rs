use super::*;
use crate::{
    game::calendar::GameCalendar,
    level::{CurrentCase, PausedInteractRequested, RoadEffect, RoadId, SelectedCase},
    test_support::{assert_absent, case_def, named, single},
};
use bevy_replique::prelude::{
    DialogueChoices, DialogueCommand, DialogueFinished, DialogueLine, ResumeDialogue, ResumeInput,
};
use std::time::{Duration, Instant};

#[test]
fn paused_buildings_start_dialogue_without_running_their_effects() {
    for (building, node) in [
        ("ConvenienceStore", "convenience_store"),
        ("ToolStore", "tool_store"),
        ("LotteryStore", "lottery_store"),
        ("Cat", "cat"),
    ] {
        let mut app = building_app();
        app.insert_resource(State::new(CommuteState::Paused));
        app.world_mut().resource_mut::<Stamina>().0 = 35.0;
        let entity = spawn_building(&mut app, building);
        assert_eq!(app.world().get::<Name>(entity).unwrap().as_str(), building);
        let player = spawn_player(&mut app, Vec2::new(42.0, 7.0));

        app.world_mut()
            .trigger(PausedInteractRequested { entity, player });
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
        assert_eq!(requests[0].node.as_str(), node);
    }
}

#[test]
fn court_characters_start_dialogue_only_for_paused_interactions() {
    for (name, asset, node) in [
        ("Prosecution", "dialogue/prosecution.rep", "loose_sign"),
        ("JudgeBench", "dialogue/verdict.rep", "verdict"),
        ("Defense", "dialogue/defense.rep", "loose_sign"),
    ] {
        let mut app = building_app();
        app.world_mut().resource_mut::<CurrentCase>().0 = Some(SelectedCase {
            road: RoadId::ExampleRoad,
            case: case_def(
                "loose_sign",
                RoadEffect::FallingObject,
                RoadEffect::ConstructionDetour,
            ),
        });
        app.world_mut()
            .spawn_scene(crate::level::scene::road::court_scene(&[]))
            .unwrap();
        let player = single::<Player>(&mut app);
        let entity = named(&mut app, name);
        let world = app.world_mut();
        assert_eq!(world.get::<BuildingDialogue>(entity).unwrap().asset, asset);
        world.trigger(RunningInteractRequested { entity, player });
        world.flush();

        assert!(world.get::<DialogueRunner>(player).is_none(), "{node}");
        assert!(
            world.resource::<Messages<StartDialogue>>().is_empty(),
            "{node}"
        );

        app.insert_resource(State::new(CommuteState::Paused));
        app.world_mut()
            .trigger(PausedInteractRequested { entity, player });
        app.world_mut().flush();

        assert!(
            app.world().get::<DialogueRunner>(player).is_some(),
            "{node}"
        );
        let requests = app
            .world_mut()
            .resource_mut::<Messages<StartDialogue>>()
            .drain()
            .collect::<Vec<_>>();
        assert_eq!(requests.len(), 1, "{node}");
        assert_eq!(requests[0].runner, player, "{node}");
        assert_eq!(requests[0].node.as_str(), node);
    }
}

#[test]
fn court_participants_resolve_the_current_case_when_interacted_with() {
    for name in ["Prosecution", "Defense"] {
        let mut app = building_app();
        app.world_mut()
            .spawn_scene(crate::level::scene::road::court_scene(&[]))
            .unwrap();
        let player = single::<Player>(&mut app);
        let entity = named(&mut app, name);
        assert_eq!(app.world().get::<BuildingDialogue>(entity).unwrap().node, None);

        app.world_mut().resource_mut::<CurrentCase>().0 = Some(SelectedCase {
            road: RoadId::MarketRoad,
            case: case_def(
                "balcony_planter",
                RoadEffect::FallingObject,
                RoadEffect::ConstructionDetour,
            ),
        });
        app.world_mut()
            .trigger(PausedInteractRequested { entity, player });
        app.world_mut().flush();

        let requests = app
            .world_mut()
            .resource_mut::<Messages<StartDialogue>>()
            .drain()
            .collect::<Vec<_>>();
        assert_eq!(requests.len(), 1, "{name}");
        assert_eq!(requests[0].node, "balcony_planter", "{name}");
        assert_eq!(requests[0].runner, player, "{name}");
    }
}

#[test]
fn court_participants_without_a_case_leave_velocity_and_ui_unchanged() {
    for name in ["Prosecution", "Defense"] {
        let mut app = building_app();
        app.world_mut()
            .spawn_scene(crate::level::scene::road::court_scene(&[]))
            .unwrap();
        let player = single::<Player>(&mut app);
        let entity = named(&mut app, name);
        app.world_mut().get_mut::<LinearVelocity>(player).unwrap().0 = Vec2::new(42.0, 7.0);
        app.world_mut()
            .trigger(PausedInteractRequested { entity, player });
        app.world_mut().flush();

        assert!(app.world().get::<DialogueRunner>(player).is_none(), "{name}");
        assert_eq!(
            app.world().get::<LinearVelocity>(player).unwrap().0,
            Vec2::new(42.0, 7.0),
            "{name}",
        );
        assert!(
            app.world().resource::<Messages<StartDialogue>>().is_empty(),
            "{name}",
        );
        assert_absent::<Node>(&mut app);
    }
}

#[test]
fn every_building_dialogue_option_is_information_only() {
    for building in [
        "ConvenienceStore",
        "ToolStore",
        "LotteryStore",
        "Cat",
    ] {
        for choice in 0..2 {
            let mut app = building_app();
            app.insert_resource(State::new(CommuteState::Paused));
            app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 0;
            app.world_mut().resource_mut::<Stamina>().0 = 35.0;
            let entity = spawn_building(&mut app, building);
            let player = spawn_player(&mut app, Vec2::ZERO);
            app.world_mut()
                .trigger(PausedInteractRequested { entity, player });
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
