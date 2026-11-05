use super::*;

#[test]
fn calendar_hud_is_independent_and_refreshes_when_dates_change_while_paused() {
    let mut app = game();
    app.world_mut()
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Paused);
    app.update();
    let hud = single::<CalendarHud>(&mut app);
    let world = app.world_mut();
    assert!(!world.entity(hud).contains::<ChildOf>());
    let node = world.get::<Node>(hud).unwrap();
    assert_eq!(node.left, px(16));
    assert_eq!(node.top, px(16));
    assert_eq!(
        text::<CalendarHudText>(&mut app),
        "Days left: 28\nSaturday (Weekend)"
    );

    app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 1;
    app.update();
    assert_eq!(
        text::<CalendarHudText>(&mut app),
        "Days left: 27\nSunday (Weekend)"
    );

    app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 2;
    app.update();
    assert_eq!(text::<CalendarHudText>(&mut app), "Days left: 26\nMonday");
    assert_eq!(app.world().resource::<CommuteTime>().elapsed, 0.0);
    assert!(app.world().get_entity(hud).is_ok());
}

#[test]
fn hud_refreshes_when_each_displayed_resource_changes_while_paused() {
    let mut app = game();
    app.world_mut()
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Paused);
    app.update();
    let bar = single::<StaminaBar>(&mut app);
    app.world_mut().resource_mut::<GameSettings>().commute_limit = 20.0;
    app.update();
    assert!(text::<CommuteHudText>(&mut app).contains("Time: 20.0s"));

    app.world_mut().resource_mut::<CommuteTime>().elapsed = 21.0;
    app.update();
    assert!(text::<CommuteHudText>(&mut app).contains("Time: -1.0s"));

    app.world_mut().resource_mut::<Money>().0 = 23;
    app.update();
    assert!(text::<CommuteHudText>(&mut app).contains("Money: 23"));

    for fraction in [0.375, 0.0, 1.0] {
        let stamina = Stamina::MAX * fraction;
        app.world_mut().resource_mut::<Stamina>().0 = stamina;
        app.update();
        assert_eq!(
            app.world().get::<Node>(bar).unwrap().width,
            percent(fraction * 100.0)
        );
        assert!(text::<CommuteHudText>(&mut app).contains(&format!("Stamina: {stamina:.0}/")));
    }
}
