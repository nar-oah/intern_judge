use super::*;
use crate::{
    config::CaseDefs,
    level::{RoadId, SelectedCase},
    test_support::{assert_absent, assert_removed, scene_entities},
};
use bevy_replique::prelude::DialogueSystem;

#[test]
fn all_huds_follow_playing_lifecycle_and_remove_their_children() {
    let mut app = game();
    for _ in 0..3 {
        let roots = [
            single::<CommuteHud>(&mut app),
            single::<CalendarHud>(&mut app),
            single::<CaseHud>(&mut app),
        ];
        let mut entities = Vec::new();
        for root in roots {
            assert!(app.world().get::<ChildOf>(root).is_none());
            let descendants = scene_entities(app.world(), root);
            assert!(descendants.len() > 1);
            entities.extend(descendants);
        }

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::MainMenu);
        app.update();

        assert_removed(&app, &entities);
        assert_absent::<CommuteHud>(&mut app);
        assert_absent::<CalendarHud>(&mut app);
        assert_absent::<CaseHud>(&mut app);
        assert_absent::<CommuteHudText>(&mut app);
        assert_absent::<CalendarHudText>(&mut app);
        assert_absent::<CaseHudText>(&mut app);
        assert_absent::<StaminaBar>(&mut app);

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        assert_removed(&app, &entities);
        assert_eq!(count::<CommuteHud>(&mut app), 1);
        assert_eq!(count::<CalendarHud>(&mut app), 1);
        assert_eq!(count::<CaseHud>(&mut app), 1);
    }
}

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

fn hud_change_ticks(app: &mut App) -> [u32; 5] {
    let commute = single::<CommuteHudText>(app);
    let stamina = single::<StaminaBar>(app);
    let calendar = single::<CalendarHudText>(app);
    let case = single::<CaseHud>(app);
    let case_text = single::<CaseHudText>(app);
    let world = app.world();
    [
        world
            .entity(commute)
            .get_ref::<Text>()
            .unwrap()
            .last_changed()
            .get(),
        world
            .entity(stamina)
            .get_ref::<Node>()
            .unwrap()
            .last_changed()
            .get(),
        world
            .entity(calendar)
            .get_ref::<Text>()
            .unwrap()
            .last_changed()
            .get(),
        world
            .entity(case)
            .get_ref::<Visibility>()
            .unwrap()
            .last_changed()
            .get(),
        world
            .entity(case_text)
            .get_ref::<Text>()
            .unwrap()
            .last_changed()
            .get(),
    ]
}

#[test]
fn each_hud_updates_only_when_its_displayed_resources_change() {
    let mut app = game();
    app.world_mut()
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Paused);
    app.update();
    let initial = hud_change_ticks(&mut app);

    advance(&mut app, 3);
    assert_eq!(hud_change_ticks(&mut app), initial);

    app.world_mut().resource_mut::<Money>().0 = 41;
    app.update();
    let after_money = hud_change_ticks(&mut app);
    assert_ne!(after_money[0], initial[0]);
    assert_eq!(after_money[2..], initial[2..]);

    app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 2;
    app.update();
    let after_calendar = hud_change_ticks(&mut app);
    assert_ne!(after_calendar[2], after_money[2]);
    assert_eq!(after_calendar[..2], after_money[..2]);
    assert_eq!(after_calendar[3..], after_money[3..]);

    let case = app.world().resource::<CaseDefs>().0[0].clone();
    app.world_mut().resource_mut::<CurrentCase>().0 = Some(SelectedCase {
        road: RoadId::MarketRoad,
        case,
    });
    app.update();
    let after_case = hud_change_ticks(&mut app);
    assert_eq!(after_case[..3], after_calendar[..3]);
    assert_ne!(after_case[3], after_calendar[3]);
    assert_ne!(after_case[4], after_calendar[4]);
    assert_eq!(text::<CaseHudText>(&mut app), "今日涉案路段：MarketRoad");

    advance(&mut app, 3);
    assert_eq!(hud_change_ticks(&mut app), after_case);
}

#[test]
fn case_hud_reflects_dialogue_commands_in_the_same_frame() {
    let mut app = game();
    let case = app.world().resource::<CaseDefs>().0[0].clone();
    app.world_mut().resource_mut::<CurrentCase>().0 = Some(SelectedCase {
        road: RoadId::ExampleRoad,
        case,
    });
    app.update();
    let hud = single::<CaseHud>(&mut app);
    assert_eq!(
        app.world().get::<Visibility>(hud),
        Some(&Visibility::Visible)
    );

    app.add_systems(
        PostUpdate,
        (|mut current: ResMut<CurrentCase>| current.0 = None).in_set(DialogueSystem::Commands),
    );
    app.update();

    assert!(app.world().resource::<CurrentCase>().0.is_none());
    assert_eq!(
        app.world().get::<Visibility>(hud),
        Some(&Visibility::Hidden)
    );
    assert_eq!(text::<CaseHudText>(&mut app), "");
}
