use super::*;
use crate::test_support::{advance, finish_app};
use bevy::{state::app::StatesPlugin, time::TimeUpdateStrategy};
use std::time::Duration;

fn game() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            125,
        )))
        .add_plugins(GamePlugin);
    app
}

fn hud(app: &mut App) -> Entity {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<CommuteHud>>()
        .single(world)
        .unwrap()
}

fn hud_text(app: &mut App) -> &str {
    let world = app.world_mut();
    &world
        .query_filtered::<&Text, With<CommuteHudText>>()
        .single(world)
        .unwrap()
        .0
}

#[test]
fn remaining_time_can_be_positive_zero_or_negative() {
    for (elapsed, remaining) in [(2.0, 3.0), (5.0, 0.0), (8.0, -3.0)] {
        assert_eq!(
            CommuteTime {
                limit: 5.0,
                elapsed,
            }
            .remaining(),
            remaining
        );
    }
}

#[test]
fn running_timer_continues_after_timeout_and_pauses_until_resumed() {
    let mut app = game();
    app.world_mut().resource_mut::<CommuteTime>().limit = 0.125;
    finish_app(&mut app);
    assert_eq!(
        *app.world().resource::<State<CommuteState>>().get(),
        CommuteState::Running
    );

    advance(&mut app, 3);
    let time = app.world().resource::<CommuteTime>();
    assert_eq!(time.elapsed, 0.375);
    assert_eq!(time.remaining(), -0.25);

    app.world_mut()
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Paused);
    advance(&mut app, 4);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.375);

    app.world_mut()
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Running);
    app.update();
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.5);
    assert_eq!(app.world().resource::<CommuteTime>().remaining(), -0.375);
}

#[test]
fn plugin_preserves_existing_resources() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
        .insert_resource(CommuteTime {
            limit: 45.0,
            elapsed: 12.0,
        })
        .insert_resource(Money(-7))
        .insert_resource(Stamina(37.5))
        .insert_resource(HasHelmet(true))
        .add_plugins(GamePlugin);
    finish_app(&mut app);

    let time = app.world().resource::<CommuteTime>();
    assert_eq!(time.limit, 45.0);
    assert_eq!(time.elapsed, 12.0);
    assert_eq!(app.world().resource::<Money>().0, -7);
    assert_eq!(app.world().resource::<Stamina>().0, 37.5);
    assert!(app.world().resource::<HasHelmet>().0);
}

#[test]
fn prototype_starts_with_one_hundred_yuan_and_no_helmet() {
    let mut app = game();
    finish_app(&mut app);
    assert_eq!(app.world().resource::<Money>().0, 100);
    assert!(!app.world().resource::<HasHelmet>().0);
}

#[test]
fn hud_is_a_top_right_root_and_updates_with_time_and_money() {
    let mut app = game();
    finish_app(&mut app);
    let hud = hud(&mut app);
    let entity = app.world().entity(hud);
    assert!(!entity.contains::<ChildOf>());
    let node = entity.get::<Node>().unwrap();
    assert_eq!(node.position_type, PositionType::Absolute);
    assert_eq!(node.top, px(16));
    assert_eq!(node.right, px(16));
    assert_eq!(
        hud_text(&mut app),
        "Time: 300.0s\nMoney: 100\nStamina: 100/100"
    );

    {
        let mut time = app.world_mut().resource_mut::<CommuteTime>();
        time.limit = 1.0;
        time.elapsed = 2.0;
    }
    app.world_mut().resource_mut::<Money>().0 = 23;
    app.update();
    assert_eq!(
        hud_text(&mut app),
        "Time: -1.1s\nMoney: 23\nStamina: 100/100"
    );
}

#[test]
fn stamina_starts_full_and_does_not_regenerate() {
    let mut app = game();
    finish_app(&mut app);
    assert_eq!(Stamina::MAX, 100.0);
    assert_eq!(app.world().resource::<Stamina>().0, Stamina::MAX);

    app.world_mut().resource_mut::<Stamina>().0 = 42.0;
    advance(&mut app, 16);
    assert_eq!(app.world().resource::<Stamina>().0, 42.0);
}

#[test]
fn stamina_bar_updates_while_timer_and_money_are_unchanged() {
    let mut app = game();
    finish_app(&mut app);
    let root = hud(&mut app);
    let world = app.world_mut();
    let bar = world
        .query_filtered::<Entity, With<StaminaBar>>()
        .single(world)
        .unwrap();
    let track = world.get::<ChildOf>(bar).unwrap().parent();
    assert_eq!(world.get::<ChildOf>(track).unwrap().parent(), root);
    assert_eq!(world.get::<Node>(bar).unwrap().width, percent(100));
    assert_eq!(world.get::<Node>(track).unwrap().width, px(200));
    assert_eq!(world.get::<Node>(track).unwrap().height, px(12));

    world
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Paused);
    app.update();
    let elapsed = app.world().resource::<CommuteTime>().elapsed;
    for remaining in [75.0, 50.0, 0.0] {
        app.world_mut().resource_mut::<Stamina>().0 = remaining;
        app.update();
        assert_eq!(
            app.world().get::<Node>(bar).unwrap().width,
            percent(remaining)
        );
        assert_eq!(app.world().resource::<CommuteTime>().elapsed, elapsed);
        assert_eq!(app.world().resource::<Money>().0, 100);
        assert!(hud_text(&mut app).ends_with(&format!("Stamina: {remaining:.0}/100")));
    }
}
