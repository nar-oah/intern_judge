use super::*;

#[test]
fn new_game_uses_configured_money_and_starts_with_full_stamina_and_running_time() {
    let mut app = game();
    assert_eq!(
        app.world().resource::<Money>().0,
        app.world().resource::<GameSettings>().initial_money
    );
    assert_eq!(app.world().resource::<Stamina>().0, Stamina::MAX);
    let calendar = app.world().resource::<GameCalendar>();
    assert_eq!(
        calendar.total_days,
        app.world().resource::<GameSettings>().total_days
    );
    assert_eq!(calendar.elapsed_days, 0);
    assert_eq!(calendar.weekday(), 6);
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
    app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 4;
    app.world_mut()
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Paused);
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::MainMenu);
    app.update();
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 4.5);

    app.world_mut().resource_mut::<GameSettings>().initial_money = 91;
    app.world_mut().resource_mut::<GameSettings>().total_days = 14;
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    advance(&mut app, 2);

    assert_eq!(app.world().resource::<Money>().0, 91);
    assert_eq!(app.world().resource::<Stamina>().0, Stamina::MAX);
    assert!(!app.world().resource::<HasHelmet>().0);
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.0);
    let calendar = app.world().resource::<GameCalendar>();
    assert_eq!(calendar.total_days, 14);
    assert_eq!(calendar.elapsed_days, 0);
    assert_eq!(
        *app.world().resource::<State<CommuteState>>().get(),
        CommuteState::Running
    );
    assert_eq!(count::<CalendarHud>(&mut app), 1);
    assert_eq!(count::<CommuteHud>(&mut app), 1);
    assert_eq!(count::<CaseHud>(&mut app), 1);
}
