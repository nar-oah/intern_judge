use super::*;

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
