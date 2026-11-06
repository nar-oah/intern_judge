use super::*;

#[test]
fn verdict_applies_effects_then_waits_for_dialogue_cleanup_before_resetting() {
    for elapsed_days in [2, 6] {
        let (mut app, player, _) = court_app();
        let selected = app.world().resource::<CurrentCase>().0.clone().unwrap();
        app.world_mut().resource_mut::<GameCalendar>().elapsed_days = elapsed_days;
        let balance = app.world().resource::<Money>().0;
        let root = single::<SceneRoot>(&mut app);
        let source = "\
:= start
>> verdict(true)
判决后的对话
=> END
---
";
        start_fixture(&mut app, player, source);
        let hud = named(&mut app, "CaseHudRoot");
        assert_eq!(
            app.world().get::<Visibility>(hud),
            Some(&Visibility::Hidden)
        );
        advance(&mut app, 4);

        assert_prompt(&mut app, "判决后的对话");
        assert!(app.world().get_entity(root).is_ok());
        assert!(app.world().get::<DialogueRunner>(player).is_some());
        assert!(app.world().contains_resource::<DayEndRequested>());
        assert!(app.world().resource::<CurrentCase>().0.is_none());
        assert_eq!(
            app.world().resource::<GameCalendar>().elapsed_days,
            elapsed_days
        );
        assert_eq!(
            app.world().resource::<RoadEffects>().0[&selected.road],
            [selected.case.guilty]
        );

        press_e(&mut app);
        advance(&mut app, 4);
        assert_closed(&mut app, player);
        assert!(app.world().get_entity(root).is_err());
        assert!(!app.world().contains_resource::<DayEndRequested>());
        if elapsed_days == 6 {
            single::<WeeklyBill>(&mut app);
            assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 6);
            assert_eq!(app.world().resource::<Money>().0, balance);
            press_e(&mut app);
            advance(&mut app, 4);
        }
        assert_eq!(
            app.world().resource::<GameCalendar>().elapsed_days,
            elapsed_days + 1
        );
        assert_eq!(
            app.world().resource::<State<GameState>>().get(),
            &GameState::Playing
        );
    }
}

#[test]
fn leaving_court_on_friday_closes_dialogue_without_billing_or_advancing() {
    let (mut app, player, bench) = court_app();
    let selected = app.world().resource::<CurrentCase>().0.clone().unwrap();
    app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 6;
    let balance = app.world().resource::<Money>().0;
    approach(&mut app, player, bench);
    press_e(&mut app);
    wait_for_dialogue(&mut app, false);
    press_e(&mut app);
    wait_for_dialogue(&mut app, true);
    select(&mut app, 2);

    assert_closed(&mut app, player);
    assert!(app.world().get_entity(player).is_ok());
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 6);
    assert_eq!(app.world().resource::<Money>().0, balance);
    assert!(!app.world().contains_resource::<DayEndRequested>());
    assert_current_case(&app, &selected);
    assert_absent::<WeeklyBill>(&mut app);
}

#[test]
fn final_friday_verdict_requires_bill_confirmation_before_ending_the_internship() {
    let (mut app, player, bench) = court_app();
    let selected = app.world().resource::<CurrentCase>().0.clone().unwrap();
    app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 27;
    approach(&mut app, player, bench);
    press_e(&mut app);
    wait_for_dialogue(&mut app, false);
    press_e(&mut app);
    wait_for_dialogue(&mut app, true);
    select(&mut app, 1);

    assert_closed(&mut app, player);
    assert!(app.world().get_entity(player).is_err());
    let balance = app.world().resource::<Money>().0;
    let settings = app.world().resource::<crate::config::GameSettings>();
    let payment = settings.weekly_salary - settings.weekly_rent;
    let bill = single::<WeeklyBill>(&mut app);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 27);
    press_e(&mut app);
    advance(&mut app, 4);
    assert!(app.world().get_entity(bill).is_err());
    assert_eq!(app.world().resource::<Money>().0, balance + payment);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 28);
    single::<InternshipEnded>(&mut app);
    assert_eq!(
        app.world().resource::<RoadEffects>().0[&selected.road],
        [selected.case.not_guilty]
    );
    assert!(app.world().resource::<CurrentCase>().0.is_none());
    assert_eq!(
        app.world().resource::<State<CommuteState>>().get(),
        &CommuteState::Paused
    );
}
