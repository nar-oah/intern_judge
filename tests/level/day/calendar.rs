use super::support::calendar_hud;
use crate::{
    game::calendar::GameCalendar,
    level::{
        RoadId, SwitchPoint,
        day::begin_next_day,
        navigation::{open_destinations, select_destination},
        tests::support::{assert_road, destination_button, weekday_app},
    },
    player::Player,
    test_support::{assert_absent, single},
};

#[test]
fn calendar_hud_survives_road_travel_and_refreshes_after_the_next_day() {
    let mut app = weekday_app(28);
    let (hud, text, initial_date) = calendar_hud(&mut app);
    let player = single::<Player>(&mut app);
    let point = single::<SwitchPoint>(&mut app);
    open_destinations(app.world_mut(), point, player);
    assert_absent::<Player>(&mut app);
    assert_eq!(calendar_hud(&mut app), (hud, text, initial_date.clone()));
    let button = destination_button(&mut app, RoadId::MarketRoad);
    select_destination(app.world_mut(), button);
    app.update();
    assert_road(&mut app, RoadId::MarketRoad);
    assert_eq!(calendar_hud(&mut app), (hud, text, initial_date.clone()));

    begin_next_day(app.world_mut());
    app.update();

    let (new_hud, new_text, next_date) = calendar_hud(&mut app);
    assert_eq!(new_hud, hud);
    assert_eq!(new_text, text);
    assert_ne!(next_date, initial_date);
    assert_eq!(app.world().resource::<GameCalendar>().elapsed_days, 3);
    assert_road(&mut app, RoadId::ExampleRoad);
}
