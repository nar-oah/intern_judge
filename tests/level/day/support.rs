use crate::{
    game::{CommuteState, calendar::GameCalendar},
    level::{
        RoadId, SceneRoot,
        tests::support::{assert_commute, assert_road, assert_today_roads},
        ui::TravelUiRoot,
    },
    player::Player,
    test_support::{assert_absent, single},
};
use bevy::prelude::*;

pub(super) fn assert_day_scene(app: &mut App) {
    if app.world().resource::<GameCalendar>().is_weekend() {
        single::<TravelUiRoot>(app);
        assert_absent::<Player>(app);
        assert_today_roads(app, &[]);
        assert_commute(app, CommuteState::Paused);
    } else {
        assert_road(app, RoadId::ExampleRoad);
        assert_today_roads(app, &[RoadId::ExampleRoad]);
        assert_commute(app, CommuteState::Running);
    }
}

pub(super) fn calendar_hud(app: &mut App) -> (Entity, Entity, String) {
    let world = app.world_mut();
    let (text_entity, text) = world
        .query::<(Entity, &Text)>()
        .iter(world)
        .find_map(|(entity, text)| {
            text.0
                .starts_with("Days left: ")
                .then_some((entity, text.0.clone()))
        })
        .expect("the calendar HUD contains its date text");
    let root = world.get::<ChildOf>(text_entity).unwrap().parent();
    assert!(world.get::<ChildOf>(root).is_none());
    assert!(world.get::<SceneRoot>(root).is_none());
    let node = world.get::<Node>(root).unwrap();
    assert_eq!(node.left, px(16));
    assert_eq!(node.top, px(16));
    (root, text_entity, text)
}
