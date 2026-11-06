use crate::{
    config::RoadGraph,
    game::{CommuteState, calendar::GameCalendar},
    level::{
        CurrentCase, RoadId, RoadUnlocks, SceneRoot, SwitchPoint,
        map::MapLayout,
        navigation,
        tests::support::{
            activate, assert_commute, assert_map, assert_road, assert_today_roads,
            destination_button, navigation_app, open_destinations, open_unlock_map,
            select_destination,
        },
        ui::{TravelButtonAction, TravelUiRoot},
    },
    player::Player,
    test_support::{assert_absent, assert_removed, scene_entities, single},
};
use bevy::{prelude::*, ui_widgets::Activate};

#[test]
fn missing_and_empty_connections_offer_no_travel() {
    let mut app = navigation_app();
    let road = single::<SceneRoot>(&mut app);
    let player = single::<Player>(&mut app);
    let point = single::<SwitchPoint>(&mut app);
    for connections in [None, Some(vec![])] {
        let mut graph = app.world_mut().resource_mut::<RoadGraph>();
        match connections {
            Some(connections) => {
                graph.roads.insert(RoadId::ExampleRoad, connections);
            }
            None => {
                graph.roads.remove(&RoadId::ExampleRoad);
            }
        }
        open_destinations(app.world_mut(), point);
        assert_eq!(single::<SceneRoot>(&mut app), road);
        assert_eq!(single::<Player>(&mut app), player);
    }
}

#[test]
fn weekday_navigation_only_offers_unlocked_neighbors_and_rechecks_before_travel() {
    for unavailable in [Some(false), None] {
        let mut app = navigation_app();
        {
            let mut unlocks = app.world_mut().resource_mut::<RoadUnlocks>();
            unlocks.0.remove(&RoadId::MarketRoad);
            if let Some(unlocked) = unavailable {
                unlocks.0.insert(RoadId::MarketRoad, unlocked);
            }
            unlocks.0.insert(RoadId::EmptyRoad1, true);
        }
        let point = single::<SwitchPoint>(&mut app);
        open_destinations(app.world_mut(), point);
        let root = assert_map(&mut app, &[RoadId::Court]);
        let button = destination_button(&mut app, RoadId::Court);
        {
            let mut unlocks = app.world_mut().resource_mut::<RoadUnlocks>();
            unlocks.0.remove(&RoadId::Court);
            if let Some(unlocked) = unavailable {
                unlocks.0.insert(RoadId::Court, unlocked);
            }
        }
        select_destination(app.world_mut(), button);
        assert_eq!(single::<SceneRoot>(&mut app), root);
        assert_absent::<Player>(&mut app);
        assert_today_roads(&app, &[RoadId::ExampleRoad]);
        assert!(app.world().resource::<CurrentCase>().0.is_none());
    }
}

#[test]
fn weekday_with_no_unlocked_neighbors_keeps_the_current_road() {
    let mut app = navigation_app();
    app.world_mut().resource_mut::<RoadUnlocks>().0 = [
        (RoadId::ExampleRoad, true),
        (RoadId::MarketRoad, false),
        (RoadId::Court, false),
    ]
    .into();
    let root = single::<SceneRoot>(&mut app);
    let player = single::<Player>(&mut app);
    let point = single::<SwitchPoint>(&mut app);
    open_destinations(app.world_mut(), point);
    assert_eq!(single::<SceneRoot>(&mut app), root);
    assert_eq!(single::<Player>(&mut app), player);
    assert_absent::<TravelUiRoot>(&mut app);
}

#[test]
fn weekend_unlocks_only_the_frontier_and_enters_the_selected_road_paused() {
    let mut app = navigation_app();
    app.world_mut().resource_mut::<GameCalendar>().elapsed_days = 0;
    app.world_mut().resource_mut::<RoadGraph>().roads = [
        (
            RoadId::ExampleRoad,
            vec![RoadId::MarketRoad, RoadId::MarketRoad, RoadId::Court],
        ),
        (RoadId::MarketRoad, vec![RoadId::EmptyRoad1]),
        (RoadId::EmptyRoad1, vec![RoadId::MarketRoad]),
        (RoadId::Court, vec![]),
    ]
    .into();
    app.world_mut().resource_mut::<RoadUnlocks>().0 = [
        (RoadId::ExampleRoad, true),
        (RoadId::MarketRoad, false),
        (RoadId::Court, true),
        (RoadId::EmptyRoad1, false),
    ]
    .into();
    let layout = MapLayout::from_graph(app.world().resource::<RoadGraph>());
    app.insert_resource(layout);
    let old_root = single::<SceneRoot>(&mut app);
    let old_scene = scene_entities(app.world(), old_root);
    let extra_root = app.world_mut().spawn(SceneRoot).id();
    let extra_child = app.world_mut().spawn(ChildOf(extra_root)).id();
    let unrelated = app.world_mut().spawn_empty().id();

    open_unlock_map(app.world_mut());

    assert_removed(&app, &old_scene);
    assert_removed(&app, &[extra_root, extra_child]);
    assert!(app.world().get_entity(unrelated).is_ok());
    assert_map(&mut app, &[RoadId::MarketRoad]);
    assert_today_roads(&app, &[RoadId::ExampleRoad]);
    assert!(!app.world().resource::<RoadUnlocks>().0[&RoadId::MarketRoad]);
    activate(&mut app, RoadId::MarketRoad);
    assert_road(&mut app, RoadId::MarketRoad);
    assert_today_roads(&app, &[RoadId::ExampleRoad, RoadId::MarketRoad]);
    assert!(app.world().resource::<RoadUnlocks>().0[&RoadId::MarketRoad]);
    assert!(app.world().resource::<RoadUnlocks>().0[&RoadId::Court]);
    assert!(!app.world().resource::<RoadUnlocks>().0[&RoadId::EmptyRoad1]);
    app.update();
    assert_commute(&app, CommuteState::Paused);

    open_unlock_map(app.world_mut());
    assert_map(&mut app, &[RoadId::EmptyRoad1]);
    assert_today_roads(&app, &[RoadId::ExampleRoad, RoadId::MarketRoad]);
    activate(&mut app, RoadId::EmptyRoad1);
    assert_road(&mut app, RoadId::EmptyRoad1);
    assert_today_roads(
        &app,
        &[RoadId::ExampleRoad, RoadId::MarketRoad, RoadId::EmptyRoad1],
    );
    assert!(app.world().resource::<RoadUnlocks>().0[&RoadId::EmptyRoad1]);
    assert!(app.world().resource::<RoadUnlocks>().0[&RoadId::Court]);
    app.update();
    assert_commute(&app, CommuteState::Paused);
}

#[test]
fn duplicate_connections_offer_each_forward_destination_once() {
    let mut app = navigation_app();
    app.world_mut().resource_mut::<RoadGraph>().roads.insert(
        RoadId::ExampleRoad,
        vec![
            RoadId::MarketRoad,
            RoadId::MarketRoad,
            RoadId::Court,
            RoadId::Court,
        ],
    );
    let point = single::<SwitchPoint>(&mut app);
    open_destinations(app.world_mut(), point);
    assert_map(&mut app, &[RoadId::MarketRoad, RoadId::Court]);
    activate(&mut app, RoadId::MarketRoad);
    assert_road(&mut app, RoadId::MarketRoad);
}

#[test]
fn navigation_rejects_destinations_without_a_map_or_outside_its_choices() {
    let mut app = navigation_app();
    let road = single::<SceneRoot>(&mut app);
    let player = single::<Player>(&mut app);
    let point = single::<SwitchPoint>(&mut app);
    let orphan = app
        .world_mut()
        .spawn(TravelButtonAction(RoadId::Court))
        .observe(navigation::select_destination)
        .id();
    select_destination(app.world_mut(), orphan);
    assert_eq!(app.world().get::<RoadId>(road), Some(&RoadId::ExampleRoad));
    assert!(app.world().get_entity(player).is_ok());
    assert_absent::<TravelUiRoot>(&mut app);
    assert_today_roads(&app, &[RoadId::ExampleRoad]);
    assert!(app.world().resource::<CurrentCase>().0.is_none());
    app.world_mut().despawn(orphan);

    open_destinations(app.world_mut(), point);
    let map = assert_map(&mut app, &[RoadId::MarketRoad, RoadId::Court]);
    let button = destination_button(&mut app, RoadId::MarketRoad);
    app.world_mut()
        .get_mut::<TravelButtonAction>(button)
        .unwrap()
        .0 = RoadId::ExampleRoad;
    select_destination(app.world_mut(), button);
    assert_eq!(single::<SceneRoot>(&mut app), map);
    assert_absent::<Player>(&mut app);
    assert_today_roads(&app, &[RoadId::ExampleRoad]);
    assert!(app.world().resource::<CurrentCase>().0.is_none());
}

#[test]
fn duplicate_and_stale_destination_selections_cannot_navigate_a_replacement_scene() {
    let mut app = navigation_app();
    let point = single::<SwitchPoint>(&mut app);
    open_destinations(app.world_mut(), point);
    let map = assert_map(&mut app, &[RoadId::MarketRoad, RoadId::Court]);
    let market = destination_button(&mut app, RoadId::MarketRoad);
    let stale_court = destination_button(&mut app, RoadId::Court);
    for button in [market, stale_court, market] {
        app.world_mut()
            .commands()
            .trigger(Activate { entity: button });
    }
    app.world_mut().flush();
    let (road, _) = assert_road(&mut app, RoadId::MarketRoad);
    assert!(app.world().get_entity(map).is_err());
    app.update();
    assert_commute(&app, CommuteState::Running);
    select_destination(app.world_mut(), stale_court);
    assert_eq!(single::<SceneRoot>(&mut app), road);

    let next_point = single::<SwitchPoint>(&mut app);
    open_destinations(app.world_mut(), next_point);
    let next_map = assert_map(&mut app, &[RoadId::Court]);
    select_destination(app.world_mut(), stale_court);
    assert_eq!(single::<SceneRoot>(&mut app), next_map);
    activate(&mut app, RoadId::Court);
    assert_road(&mut app, RoadId::Court);
    app.update();
    assert_commute(&app, CommuteState::Paused);
}
