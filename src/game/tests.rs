use super::*;
use crate::test_support::{advance, finish_app, insert_config};
use bevy::{state::app::StatesPlugin, time::TimeUpdateStrategy};
use std::time::Duration;

fn game() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin));
    insert_config(&mut app);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        125,
    )))
    .add_plugins(GamePlugin);
    finish_app(&mut app);
    app
}

fn hud_text(app: &mut App) -> String {
    let world = app.world_mut();
    world
        .query_filtered::<&Text, With<CommuteHudText>>()
        .single(world)
        .unwrap()
        .0
        .clone()
}

#[test]
fn new_game_uses_configured_money_and_starts_with_full_stamina_and_running_time() {
    let mut app = game();
    assert_eq!(
        app.world().resource::<Money>().0,
        app.world().resource::<GameSettings>().initial_money
    );
    assert_eq!(app.world().resource::<Stamina>().0, Stamina::MAX);
    assert!(!app.world().resource::<HasHelmet>().0);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.0);
    assert_eq!(
        *app.world().resource::<State<CommuteState>>().get(),
        CommuteState::Running
    );

    app.world_mut().resource_mut::<Stamina>().0 = 12.0;
    advance(&mut app, 4);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.5);
    assert_eq!(app.world().resource::<Stamina>().0, 12.0);
}

#[test]
fn entering_playing_resets_session_data_and_resumes_commute() {
    let mut app = game();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    app.world_mut().resource_mut::<Money>().0 = 1;
    app.world_mut().resource_mut::<Stamina>().0 = 12.0;
    app.world_mut().resource_mut::<HasHelmet>().0 = true;
    app.world_mut().resource_mut::<CommuteTime>().elapsed = 4.5;
    app.world_mut()
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Paused);
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::MainMenu);
    app.update();
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 4.5);

    app.world_mut().resource_mut::<GameSettings>().initial_money = 91;
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    advance(&mut app, 2);

    assert_eq!(app.world().resource::<Money>().0, 91);
    assert_eq!(app.world().resource::<Stamina>().0, Stamina::MAX);
    assert!(!app.world().resource::<HasHelmet>().0);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.0);
    assert_eq!(
        *app.world().resource::<State<CommuteState>>().get(),
        CommuteState::Running
    );
}

#[test]
fn commute_counts_only_running_gameplay_and_continues_past_the_limit() {
    let mut app = game();
    app.world_mut().resource_mut::<GameSettings>().commute_limit = 0.25;
    advance(&mut app, 3);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.375);
    assert_eq!(
        app.world().resource::<CommuteTime>().remaining(0.25),
        -0.125
    );

    app.world_mut()
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Paused);
    advance(&mut app, 2);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.375);

    app.world_mut()
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Running);
    app.update();
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.5);

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::MainMenu);
    advance(&mut app, 2);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.5);
}

#[test]
fn hud_refreshes_when_each_displayed_resource_changes_while_paused() {
    let mut app = game();
    app.world_mut()
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Paused);
    app.update();
    let world = app.world_mut();
    let bar = world
        .query_filtered::<Entity, With<StaminaBar>>()
        .single(world)
        .unwrap();

    world.resource_mut::<GameSettings>().commute_limit = 20.0;
    app.update();
    assert!(hud_text(&mut app).contains("Time: 20.0s"));

    app.world_mut().resource_mut::<CommuteTime>().elapsed = 21.0;
    app.update();
    assert!(hud_text(&mut app).contains("Time: -1.0s"));

    app.world_mut().resource_mut::<Money>().0 = 23;
    app.update();
    assert!(hud_text(&mut app).contains("Money: 23"));

    for fraction in [0.375, 0.0, 1.0] {
        let stamina = Stamina::MAX * fraction;
        app.world_mut().resource_mut::<Stamina>().0 = stamina;
        app.update();
        assert_eq!(
            app.world().get::<Node>(bar).unwrap().width,
            percent(fraction * 100.0)
        );
        assert!(hud_text(&mut app).contains(&format!("Stamina: {stamina:.0}/")));
    }
}
