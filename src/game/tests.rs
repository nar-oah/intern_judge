use super::*;
use crate::test_support::{advance, finish_app};
use bevy::{state::app::StatesPlugin, time::TimeUpdateStrategy};
use std::time::Duration;

fn game() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(125)))
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
        .add_plugins(GamePlugin);
    finish_app(&mut app);

    let time = app.world().resource::<CommuteTime>();
    assert_eq!(time.limit, 45.0);
    assert_eq!(time.elapsed, 12.0);
    assert_eq!(app.world().resource::<Money>().0, -7);
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
    assert_eq!(entity.get::<Text>().unwrap().0, "Time: 300.0s\nMoney: 0");

    {
        let mut time = app.world_mut().resource_mut::<CommuteTime>();
        time.limit = 1.0;
        time.elapsed = 2.0;
    }
    app.world_mut().resource_mut::<Money>().0 = 23;
    app.update();
    assert_eq!(
        app.world().get::<Text>(hud).unwrap().0,
        "Time: -1.1s\nMoney: 23"
    );
}
