use super::{ui::DialogueChoiceButton, *};
use crate::{
    config::GameState,
    game::{CommuteState, GameCalendar, GamePlugin, Stamina},
    level::{
        InitialRoad, InteractRequested, Interactable, InternshipEnded, LevelPlugin, RoadEffect,
        RoadEffects, RoadId, SceneRoot,
    },
    player::{Player, PlayerPlugin},
    test_support::{advance, finish_app, headless_app},
};
use avian2d::prelude::{Collider, CollidingEntities, LinearVelocity, Position, Sensor, ShapeHits};
use bevy::{
    asset::io::embedded::EmbeddedAssetRegistry,
    ui_widgets::{Activate, Button},
};
use leafwing_input_manager::prelude::*;
use std::{
    collections::HashMap,
    path::Path,
    time::{Duration, Instant},
};

#[derive(Resource, Default)]
struct CommandsRun(Vec<DialogueCommand>);

#[derive(Resource, Default)]
struct OtherInteractions(usize);

fn single<C: Component>(app: &mut App) -> Entity {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<C>>()
        .single(world)
        .unwrap()
}

fn named(app: &mut App, expected: &str) -> Entity {
    let world = app.world_mut();
    world
        .query::<(Entity, &Name)>()
        .iter(world)
        .find_map(|(entity, name)| (name.as_str() == expected).then_some(entity))
        .unwrap()
}

fn court_app() -> (App, Entity, Entity) {
    let mut app = headless_app(Vec2::ZERO);
    app.init_asset::<Font>()
        .init_asset_loader::<bevy::text::FontLoader>()
        .insert_resource(InitialRoad(RoadId::Court))
        .init_resource::<CommandsRun>()
        .init_resource::<OtherInteractions>()
        .add_plugins((GamePlugin, PlayerPlugin, LevelPlugin))
        .add_systems(
            PostUpdate,
            (|mut messages: MessageReader<DialogueCommand>, mut ran: ResMut<CommandsRun>| {
                ran.0.extend(messages.read().cloned());
            })
            .after(DialogueSystem::Commands),
        );
    finish_app(&mut app);
    let player = single::<Player>(&mut app);
    let bench = named(&mut app, "JudgeBench");
    (app, player, bench)
}

fn approach(app: &mut App, player: Entity, target: Entity) {
    // Place the player on the court's floor, inside the real building sensor.
    let center = app
        .world()
        .get::<GlobalTransform>(target)
        .unwrap()
        .translation();
    let position = Vec2::new(center.x, -137.5);
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = position.extend(0.0);
    app.world_mut().get_mut::<Position>(player).unwrap().0 = position;
    app.world_mut().get_mut::<LinearVelocity>(player).unwrap().0 = Vec2::ZERO;
    advance(app, 3);
    assert!(
        app.world()
            .get::<CollidingEntities>(target)
            .unwrap()
            .contains(&player)
    );
}

fn press_e(app: &mut App) {
    KeyCode::KeyE.release(app.world_mut());
    app.update();
    KeyCode::KeyE.press(app.world_mut());
    app.update();
}

fn wait_for_dialogue(app: &mut App, choices: bool) -> Entity {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let world = app.world_mut();
        if let Some(entity) = world
            .query::<(Entity, &DialogueBox)>()
            .iter(world)
            .find_map(|(entity, dialog)| {
                let ready = if choices {
                    dialog.choices.is_some()
                } else {
                    dialog.line.is_some()
                };
                ready.then_some(entity)
            })
        {
            return entity;
        }
        app.update();
        assert!(Instant::now() < deadline, "court dialogue did not load");
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn assert_prompt(app: &mut App, expected: &str) {
    let prompt = single::<DialoguePrompt>(app);
    assert_eq!(app.world().get::<Text>(prompt).unwrap().0, expected);
}

fn buttons(app: &mut App) -> Vec<(usize, String, Entity)> {
    let world = app.world_mut();
    let mut buttons = world
        .query::<(Entity, &DialogueChoiceButton, &Children)>()
        .iter(world)
        .map(|(entity, choice, children)| {
            assert!(world.get::<Button>(entity).is_some());
            let text = world.get::<Text>(children[0]).unwrap().0.clone();
            (choice.0, text, entity)
        })
        .collect::<Vec<_>>();
    buttons.sort_by_key(|(index, _, _)| *index);
    buttons
}

fn select(app: &mut App, index: usize) {
    let button = buttons(app)
        .into_iter()
        .find(|(choice, _, _)| *choice == index)
        .unwrap()
        .2;
    // The observer must consume this suspension once even if activation repeats.
    app.world_mut().trigger(Activate { entity: button });
    app.world_mut().trigger(Activate { entity: button });
    advance(app, 4);
}

fn assert_closed(app: &mut App, player: Entity) {
    assert!(app.world().get::<DialogueRunner>(player).is_none());
    let world = app.world_mut();
    assert!(world.query::<&DialogueBox>().iter(world).next().is_none());
    assert!(
        world
            .query_filtered::<Entity, Or<(
                With<DialoguePrompt>,
                With<DialogueOptions>,
                With<DialogueChoiceButton>
            )>>()
            .iter(world)
            .next()
            .is_none()
    );
}

#[test]
fn opening_and_holding_e_keeps_the_first_line_until_a_fresh_press() {
    let (mut app, player, bench) = court_app();
    press_e(&mut app);
    assert_closed(&mut app, player);
    approach(&mut app, player, bench);
    assert_closed(&mut app, player);

    press_e(&mut app);
    assert!(app.world().get::<DialogueRunner>(player).is_some());
    let dialog = wait_for_dialogue(&mut app, false);
    advance(&mut app, 4);
    assert_prompt(&mut app, "请作出你的判决");
    assert!(
        app.world()
            .get::<DialogueBox>(dialog)
            .unwrap()
            .line
            .is_some()
    );
    assert!(
        app.world()
            .get::<DialogueBox>(dialog)
            .unwrap()
            .choices
            .is_none()
    );
    assert!(app.world().get::<ChildOf>(dialog).is_none());
    assert!(buttons(&mut app).is_empty());

    press_e(&mut app);
    wait_for_dialogue(&mut app, true);
    assert_eq!(
        buttons(&mut app)
            .into_iter()
            .map(|(index, text, _)| (index, text))
            .collect::<Vec<_>>(),
        [(0, "有罪".into()), (1, "无罪".into()), (2, "离开".into())]
    );
    let options = single::<DialogueOptions>(&mut app);
    assert!(app.world().get::<Node>(options).is_some());
    assert!(app.world().get::<Text>(options).is_none());

    // Neither E nor the old selection keys can answer a choice point.
    KeyCode::ArrowLeft.press(app.world_mut());
    KeyCode::ArrowRight.press(app.world_mut());
    press_e(&mut app);
    advance(&mut app, 4);
    assert!(
        app.world()
            .get::<DialogueBox>(dialog)
            .unwrap()
            .choices
            .is_some()
    );
    assert!(app.world().resource::<CommandsRun>().0.is_empty());

    // The UI root is independent of the player hierarchy and still cleans up.
    app.world_mut().despawn(player);
    app.world_mut().flush();
    app.update();
    assert_closed(&mut app, player);
    assert!(app.world().get_entity(bench).is_ok());
}

#[test]
fn each_court_button_runs_its_branch_and_closes_the_dialogue() {
    let (mut app, mut player, mut bench) = court_app();
    let mut expected_effects = HashMap::from([
        (RoadId::Court, vec![RoadEffect::ConstructionDetour]),
        (RoadId::MarketRoad, vec![RoadEffect::FallingObject]),
    ]);
    app.world_mut().resource_mut::<RoadEffects>().0 = expected_effects.clone();
    for (index, expected) in [
        (0, Some(true)),
        (0, Some(true)),
        (1, Some(false)),
        (1, Some(false)),
        (2, None),
    ] {
        approach(&mut app, player, bench);
        let elapsed_days = app.world().resource::<GameCalendar>().elapsed_days;
        app.world_mut().resource_mut::<CommandsRun>().0.clear();
        press_e(&mut app);
        wait_for_dialogue(&mut app, false);
        press_e(&mut app);
        wait_for_dialogue(&mut app, true);
        select(&mut app, index);
        assert_closed(&mut app, player);

        let commands = &app.world().resource::<CommandsRun>().0;
        match expected {
            Some(result) => {
                assert_eq!(commands.len(), 1);
                assert_eq!(commands[0].token.runner(), player);
                assert_eq!(commands[0].name, "verdict");
                assert_eq!(commands[0].args, [Value::Bool(result)]);
                let (road, effects) = if result {
                    (RoadId::ExampleRoad, vec![RoadEffect::FallingObject])
                } else {
                    (
                        RoadId::MarketRoad,
                        vec![RoadEffect::FallingObject, RoadEffect::ConstructionDetour],
                    )
                };
                expected_effects.insert(road, effects);
            }
            None => assert!(commands.is_empty(), "leaving must not issue a verdict"),
        }
        assert_eq!(app.world().resource::<RoadEffects>().0, expected_effects);
        advance(&mut app, 4);
        assert_closed(&mut app, player);
        assert_eq!(
            app.world().resource::<GameCalendar>().elapsed_days,
            elapsed_days + u32::from(expected.is_some())
        );
        if expected.is_some() {
            assert!(app.world().get_entity(player).is_err());
            assert!(app.world().get_entity(bench).is_err());
            player = single::<Player>(&mut app);
            bench = named(&mut app, "JudgeBench");
        } else {
            assert!(app.world().get_entity(player).is_ok());
            assert!(app.world().get_entity(bench).is_ok());
        }
    }
}

#[test]
fn verdict_applies_effects_then_waits_for_dialogue_cleanup_before_resetting() {
    let (mut app, player, _) = court_app();
    let root = single::<SceneRoot>(&mut app);
    let source = "\
:= start
>> verdict(true)
判决后的对话
=> END
---
";
    app.world()
        .resource::<EmbeddedAssetRegistry>()
        .insert_asset(
            "after-verdict.rep".into(),
            Path::new("after-verdict.rep"),
            source.as_bytes().to_vec(),
        );
    let dialogue = app
        .world()
        .resource::<AssetServer>()
        .load("embedded://after-verdict.rep");
    app.world_mut()
        .entity_mut(player)
        .insert(DialogueRunner::new(dialogue));
    app.world_mut()
        .spawn_scene(ui::dialogue_box_scene())
        .unwrap();
    app.world_mut().write_message(StartDialogue {
        runner: player,
        node: "start".into(),
    });
    wait_for_dialogue(&mut app, false);
    advance(&mut app, 4);

    assert_prompt(&mut app, "判决后的对话");
    assert!(app.world().get_entity(root).is_ok());
    assert!(app.world().get::<DialogueRunner>(player).is_some());
    assert!(app.world().contains_resource::<DayEndRequested>());
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 0);
    assert_eq!(
        app.world().resource::<RoadEffects>().0[&RoadId::ExampleRoad],
        [RoadEffect::FallingObject]
    );

    press_e(&mut app);
    advance(&mut app, 4);
    assert_closed(&mut app, player);
    assert!(app.world().get_entity(root).is_err());
    assert!(!app.world().contains_resource::<DayEndRequested>());
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 1);
    assert_eq!(
        app.world().resource::<State<GameState>>().get(),
        &GameState::Playing
    );
}

#[test]
fn final_verdict_closes_dialogue_and_ends_the_internship() {
    let (mut app, player, bench) = court_app();
    app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 27;
    approach(&mut app, player, bench);
    press_e(&mut app);
    wait_for_dialogue(&mut app, false);
    press_e(&mut app);
    wait_for_dialogue(&mut app, true);
    select(&mut app, 1);

    assert_closed(&mut app, player);
    assert!(app.world().get_entity(player).is_err());
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 28);
    single::<InternshipEnded>(&mut app);
    assert_eq!(
        app.world().resource::<RoadEffects>().0[&RoadId::MarketRoad],
        [RoadEffect::ConstructionDetour]
    );
    assert_eq!(
        app.world().resource::<State<CommuteState>>().get(),
        &CommuteState::Paused
    );
}

#[test]
fn court_participants_use_nearby_e_dialogue_and_restore_movement() {
    for (name, expected) in [
        ("Prosecution", "公诉方：请求判决有罪。"),
        ("Defense", "辩护方：请求判决无罪。"),
    ] {
        let (mut app, player, _) = court_app();
        let participant = named(&mut app, name);
        press_e(&mut app);
        assert_closed(&mut app, player);
        approach(&mut app, player, participant);
        assert_closed(&mut app, player);

        KeyCode::KeyD.press(app.world_mut());
        app.update();
        assert!(app.world().get::<LinearVelocity>(player).unwrap().x > 0.0);
        press_e(&mut app);
        wait_for_dialogue(&mut app, false);
        assert_prompt(&mut app, expected);
        advance(&mut app, 4);
        assert_prompt(&mut app, expected);
        assert_eq!(app.world().get::<LinearVelocity>(player).unwrap().x, 0.0);
        assert!(buttons(&mut app).is_empty());

        press_e(&mut app);
        advance(&mut app, 4);
        assert_closed(&mut app, player);
        assert!(app.world().get::<LinearVelocity>(player).unwrap().x > 0.0);
        assert!(app.world().resource::<CommandsRun>().0.is_empty());
        assert!(app.world().resource::<RoadEffects>().0.is_empty());
    }
}

#[test]
fn consecutive_lines_each_need_e_and_available_buttons_keep_original_indices() {
    let (mut app, player, _) = court_app();
    let source = "\
:= start
第一句
第二句
第三句
-> 不可用 [if false]
    >> verdict(false)
    => END
-> 继续
    已选择
    >> verdict(true)
    => END
---
";
    app.world()
        .resource::<EmbeddedAssetRegistry>()
        .insert_asset(
            "fixture.rep".into(),
            Path::new("fixture.rep"),
            source.as_bytes().to_vec(),
        );
    let dialogue = app
        .world()
        .resource::<AssetServer>()
        .load("embedded://fixture.rep");
    app.world_mut()
        .entity_mut(player)
        .insert(DialogueRunner::new(dialogue));
    app.world_mut()
        .spawn_scene(ui::dialogue_box_scene())
        .unwrap();
    app.world_mut().write_message(StartDialogue {
        runner: player,
        node: "start".into(),
    });
    wait_for_dialogue(&mut app, false);

    for expected in ["第一句", "第二句", "第三句"] {
        assert_prompt(&mut app, expected);
        advance(&mut app, 4);
        assert_prompt(&mut app, expected);
        assert!(buttons(&mut app).is_empty());
        press_e(&mut app);
    }
    wait_for_dialogue(&mut app, true);
    let choices = buttons(&mut app);
    assert_eq!(choices.len(), 1);
    assert_eq!((&choices[0].0, choices[0].1.as_str()), (&1, "继续"));
    let old_button = choices[0].2;
    let old_label = app.world().get::<Children>(old_button).unwrap()[0];
    select(&mut app, 1);
    wait_for_dialogue(&mut app, false);
    assert_prompt(&mut app, "已选择");
    assert!(buttons(&mut app).is_empty());
    assert!(app.world().get_entity(old_button).is_err());
    assert!(app.world().get_entity(old_label).is_err());
    assert!(app.world().resource::<CommandsRun>().0.is_empty());
    advance(&mut app, 4);
    assert_prompt(&mut app, "已选择");

    press_e(&mut app);
    advance(&mut app, 4);
    assert_closed(&mut app, player);
    let commands = &app.world().resource::<CommandsRun>().0;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].args, [Value::Bool(true)]);
}

#[test]
fn dialogue_blocks_controls_and_other_buildings_then_restores_all_input() {
    let (mut app, player, bench) = court_app();
    approach(&mut app, player, bench);
    assert!(
        app.world()
            .get::<ShapeHits>(player)
            .unwrap()
            .iter()
            .any(|hit| (-hit.normal2).dot(Vec2::Y) > 0.7)
    );

    KeyCode::KeyD.press(app.world_mut());
    KeyCode::ShiftLeft.press(app.world_mut());
    app.update();
    assert!(app.world().get::<LinearVelocity>(player).unwrap().x > 0.0);
    press_e(&mut app);
    let stamina = app.world().resource::<Stamina>().0;
    assert_eq!(app.world().get::<LinearVelocity>(player).unwrap().x, 0.0);
    KeyCode::Space.press(app.world_mut());
    wait_for_dialogue(&mut app, false);

    // Replace the nearby interaction target while the dialogue stays on the player.
    app.world_mut().entity_mut(bench).remove::<Interactable>();
    let other_building = app
        .world_mut()
        .spawn((
            Interactable,
            Collider::rectangle(240.0, 70.0),
            Sensor,
            CollidingEntities::default(),
            Transform::from_xyz(0.0, -127.5, -1.0),
        ))
        .observe(
            |_: On<InteractRequested>, mut requests: ResMut<OtherInteractions>| {
                requests.0 += 1;
            },
        )
        .id();
    advance(&mut app, 4);
    assert!(
        app.world()
            .get::<CollidingEntities>(other_building)
            .unwrap()
            .contains(&player)
    );
    press_e(&mut app);
    wait_for_dialogue(&mut app, true);
    press_e(&mut app);
    advance(&mut app, 4);
    assert_eq!(
        app.world().get::<LinearVelocity>(player).unwrap().0,
        Vec2::ZERO
    );
    assert_eq!(app.world().resource::<Stamina>().0, stamina);
    assert_eq!(app.world().resource::<OtherInteractions>().0, 0);

    select(&mut app, 2);
    assert_closed(&mut app, player);
    advance(&mut app, 2);
    assert!(app.world().get::<LinearVelocity>(player).unwrap().x > 0.0);
    assert!(app.world().resource::<Stamina>().0 < stamina);
    press_e(&mut app);
    assert_eq!(app.world().resource::<OtherInteractions>().0, 1);

    KeyCode::Space.release(app.world_mut());
    app.update();
    KeyCode::Space.press(app.world_mut());
    app.update();
    assert!(app.world().get::<LinearVelocity>(player).unwrap().y > 0.0);
}
