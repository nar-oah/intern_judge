use super::*;
use crate::{
    game::{CommuteState, Stamina},
    level::{InitialRoad, LevelPlugin, RoadId},
    player::PlayerPlugin,
    test_support::{advance, finish_app, headless_app},
};
use avian2d::prelude::{CollidingEntities, Position, ShapeHits};
use leafwing_input_manager::prelude::*;
use std::time::{Duration, Instant};

#[derive(Resource, Default)]
struct CommandsRun(Vec<DialogueCommand>);

fn single<C: Component>(app: &mut App) -> Entity {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<C>>()
        .single(world)
        .unwrap()
}

fn court_app() -> (App, Entity, Entity) {
    let mut app = headless_app(Vec2::ZERO);
    app.init_state::<CommuteState>()
        .init_asset::<Font>()
        .init_asset_loader::<bevy::text::FontLoader>()
        .insert_resource(InitialRoad(RoadId::Court))
        .init_resource::<CommandsRun>()
        .add_plugins((PlayerPlugin, LevelPlugin))
        .add_systems(
            PostUpdate,
            (|mut messages: MessageReader<DialogueCommand>, mut ran: ResMut<CommandsRun>| {
                ran.0.extend(messages.read().cloned());
            })
            .after(DialogueSystem::Commands),
        );
    finish_app(&mut app);
    let player = single::<Player>(&mut app);
    let bench = {
        let world = app.world_mut();
        world
            .query::<(Entity, &Name)>()
            .iter(world)
            .find_map(|(entity, name)| (name.as_str() == "JudgeBench").then_some(entity))
            .unwrap()
    };
    (app, player, bench)
}

fn approach(app: &mut App, player: Entity, bench: Entity) {
    // Place the player on the court's floor, inside the real bench sensor.
    let position = Vec2::new(0.0, -137.5);
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = position.extend(0.0);
    app.world_mut().get_mut::<Position>(player).unwrap().0 = position;
    app.world_mut().get_mut::<LinearVelocity>(player).unwrap().0 = Vec2::ZERO;
    advance(app, 3);
    assert!(
        app.world()
            .get::<CollidingEntities>(bench)
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

fn wait_for_choices(app: &mut App) -> Entity {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let world = app.world_mut();
        if let Some(entity) = world
            .query::<(Entity, &DialogueBox)>()
            .iter(world)
            .find_map(|(entity, dialog)| dialog.choices.is_some().then_some(entity))
        {
            return entity;
        }
        app.update();
        assert!(Instant::now() < deadline, "court dialogue did not load");
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn assert_closed(app: &mut App, player: Entity) {
    assert!(app.world().get::<DialogueRunner>(player).is_none());
    let world = app.world_mut();
    assert!(world.query::<&DialogueBox>().iter(world).next().is_none());
    assert!(
        world
            .query_filtered::<Entity, Or<(With<DialoguePrompt>, With<DialogueOptions>)>>()
            .iter(world)
            .next()
            .is_none()
    );
}

#[test]
fn bench_requires_a_fresh_interaction_and_holding_e_keeps_the_prompt_open() {
    let (mut app, player, bench) = court_app();
    press_e(&mut app);
    assert_closed(&mut app, player);

    approach(&mut app, player, bench);
    assert_closed(&mut app, player);
    press_e(&mut app);
    assert!(app.world().get::<DialogueRunner>(player).is_some());
    let dialog = wait_for_choices(&mut app);
    advance(&mut app, 4);

    let choices = app.world().get::<DialogueBox>(dialog).unwrap();
    assert_eq!(choices.runner, player);
    assert!(app.world().get::<ChildOf>(dialog).is_none());
    assert_eq!(
        choices
            .choices
            .as_ref()
            .unwrap()
            .choices
            .iter()
            .map(|choice| choice.text.as_str())
            .collect::<Vec<_>>(),
        ["有罪", "无罪", "离开"]
    );
    let prompt = single::<DialoguePrompt>(&mut app);
    assert_eq!(app.world().get::<Text>(prompt).unwrap().0, "请作出你的判决");
    let options = single::<DialogueOptions>(&mut app);
    assert_eq!(
        app.world().get::<Text>(options).unwrap().0,
        "← 左方向键：有罪    → 右方向键：无罪    E 键：离开"
    );
    assert!(app.world().resource::<CommandsRun>().0.is_empty());

    // Removing the runner's player must also clean up its independent UI root.
    app.world_mut().despawn(player);
    app.world_mut().flush();
    app.update();
    assert_closed(&mut app, player);
    assert!(app.world().get_entity(bench).is_ok());
}

#[test]
fn each_keyboard_choice_runs_its_replique_branch_and_closes_the_dialogue() {
    let (mut app, player, bench) = court_app();
    approach(&mut app, player, bench);

    for (key, expected) in [
        (KeyCode::ArrowLeft, Some("有罪")),
        (KeyCode::ArrowRight, Some("无罪")),
        (KeyCode::KeyE, None),
    ] {
        app.world_mut().resource_mut::<CommandsRun>().0.clear();
        press_e(&mut app);
        wait_for_choices(&mut app);
        KeyCode::KeyE.release(app.world_mut());
        app.update();
        key.press(app.world_mut());
        advance(&mut app, 4);
        assert_closed(&mut app, player);

        let commands = &app.world().resource::<CommandsRun>().0;
        match expected {
            Some(result) => {
                assert_eq!(commands.len(), 1);
                assert_eq!(commands[0].token.runner(), player);
                assert_eq!(commands[0].name, "verdict");
                assert_eq!(commands[0].args, [Value::String(result.into())]);
            }
            None => assert!(commands.is_empty(), "leaving must not issue a verdict"),
        }
        advance(&mut app, 4);
        assert_closed(&mut app, player);
        key.release(app.world_mut());
        app.update();
    }
}

#[test]
fn dialogue_blocks_movement_sprinting_and_jumping_then_restores_player_controls() {
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
    wait_for_choices(&mut app);
    advance(&mut app, 4);
    assert_eq!(
        app.world().get::<LinearVelocity>(player).unwrap().0,
        Vec2::ZERO
    );
    assert_eq!(app.world().resource::<Stamina>().0, stamina);

    press_e(&mut app);
    assert_closed(&mut app, player);
    advance(&mut app, 2);
    assert!(app.world().get::<LinearVelocity>(player).unwrap().x > 0.0);
    assert!(app.world().resource::<Stamina>().0 < stamina);

    KeyCode::Space.release(app.world_mut());
    app.update();
    KeyCode::Space.press(app.world_mut());
    app.update();
    assert!(app.world().get::<LinearVelocity>(player).unwrap().y > 0.0);
}
