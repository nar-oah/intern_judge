use super::*;
use crate::game::CommuteTime;

#[test]
fn verdict_applies_effects_then_waits_for_dialogue_cleanup_before_resetting() {
    for elapsed_days in [2, 6] {
        let (mut app, player, _) = court_app();
        let selected = app.world().resource::<CurrentCase>().0.clone().unwrap();
        app.world_mut().resource_mut::<GameCalendar>().elapsed_days = elapsed_days;
        app.world_mut().resource_mut::<CommuteTime>().elapsed = 4.0;
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
        assert_absent::<DailyBill>(&mut app);
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
        single::<DailyBill>(&mut app);
        assert_eq!(
            app.world().resource::<GameCalendar>().elapsed_days,
            elapsed_days
        );
        assert_eq!(app.world().resource::<Money>().0, balance);
        press_e(&mut app);
        advance(&mut app, 4);
        assert_eq!(
            app.world().resource::<GameCalendar>().elapsed_days,
            elapsed_days + 1
        );
        assert_eq!(
            app.world().resource::<State<GameState>>().get(),
            &GameState::Playing
        );
        // Four remaining seconds pay 31; Friday also charges 47 in rent.
        let payment = if elapsed_days == 6 { -16 } else { 31 };
        assert_eq!(app.world().resource::<Money>().0, balance + payment);
    }
}

#[test]
fn end_day_during_a_verdict_tail_creates_only_one_bill() {
    let (mut app, player, _) = court_app();
    let root = single::<SceneRoot>(&mut app);
    start_fixture(
        &mut app,
        player,
        ":= start\n>> verdict(true)\n判决后的对话\n=> END\n---\n",
    );
    assert_prompt(&mut app, "判决后的对话");

    app.world_mut().trigger(crate::level::day::EndDay);
    app.world_mut().flush();

    let bill = single::<DailyBill>(&mut app);
    assert_eq!(single::<SceneRoot>(&mut app), bill);
    assert!(app.world().get_entity(root).is_err());
    assert!(app.world().get_entity(player).is_err());
    assert_closed(&mut app, player);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 2);
    app.update();
    press_e(&mut app);
    advance(&mut app, 3);
    assert_absent::<DailyBill>(&mut app);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 3);
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
    assert_current_case(&app, &selected);
    assert_absent::<DailyBill>(&mut app);
}

#[test]
fn final_friday_verdict_requires_bill_confirmation_before_ending_the_internship() {
    let (mut app, player, bench) = court_app();
    let selected = app.world().resource::<CurrentCase>().0.clone().unwrap();
    app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 27;
    app.world_mut().resource_mut::<CommuteTime>().elapsed = 4.0;
    approach(&mut app, player, bench);
    press_e(&mut app);
    wait_for_dialogue(&mut app, false);
    press_e(&mut app);
    wait_for_dialogue(&mut app, true);
    select(&mut app, 1);

    assert_closed(&mut app, player);
    assert!(app.world().get_entity(player).is_err());
    let balance = app.world().resource::<Money>().0;
    let bill = single::<DailyBill>(&mut app);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 27);
    press_e(&mut app);
    advance(&mut app, 4);
    assert!(app.world().get_entity(bill).is_err());
    // Daily salary 31 minus Friday rent 47 is applied on confirmation.
    assert_eq!(app.world().resource::<Money>().0, balance - 16);
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
