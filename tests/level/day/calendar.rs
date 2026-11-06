use super::support::calendar_hud;
use crate::{
    game::calendar::GameCalendar,
    level::{
        RoadId, SwitchPoint,
        day::EndDay,
        tests::support::{
            assert_road, destination_button, open_destinations, select_destination, weekday_app,
        },
    },
    player::Player,
    test_support::{assert_absent, press_e, single},
};

#[test]
fn calendar_hud_survives_road_travel_and_refreshes_after_the_next_day() {
    let mut app = weekday_app(28);
    let (hud, text, initial_date) = calendar_hud(&mut app);
    let point = single::<SwitchPoint>(&mut app);
    open_destinations(app.world_mut(), point);
    assert_absent::<Player>(&mut app);
    assert_eq!(calendar_hud(&mut app), (hud, text, initial_date.clone()));
    let button = destination_button(&mut app, RoadId::MarketRoad);
    select_destination(app.world_mut(), button);
    app.update();
    assert_road(&mut app, RoadId::MarketRoad);
    assert_eq!(calendar_hud(&mut app), (hud, text, initial_date.clone()));

    app.world_mut().trigger(EndDay);
    app.update();
    press_e(&mut app);
    app.update();

    let (new_hud, new_text, next_date) = calendar_hud(&mut app);
    assert_eq!(new_hud, hud);
    assert_eq!(new_text, text);
    assert_ne!(next_date, initial_date);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 3);
    assert_road(&mut app, RoadId::ExampleRoad);
}
