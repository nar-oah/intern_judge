use super::*;

#[test]
fn map_displays_the_graph_but_only_outgoing_destinations_are_buttons() {
    let mut app = crate::test_support::scene_app();
    let graph = RoadGraph {
        roads: [
            (RoadId::ExampleRoad, vec![RoadId::MarketRoad]),
            (RoadId::MarketRoad, vec![RoadId::Court]),
        ]
        .into(),
        initial_unlocks: default(),
    };
    let layout = MapLayout::from_graph(&graph);
    let world = app.world_mut();
    world
        .spawn_scene(travel_ui_scene(
            Some(RoadId::ExampleRoad),
            graph.roads[&RoadId::ExampleRoad].clone(),
            &graph,
            &layout,
        ))
        .unwrap();

    let mut displayed = world
        .query::<&MapRoadNode>()
        .iter(world)
        .map(|node| node.0)
        .collect::<Vec<_>>();
    displayed.sort_unstable();
    assert_eq!(
        displayed,
        [RoadId::ExampleRoad, RoadId::MarketRoad, RoadId::Court,]
    );

    let buttons = world
        .query_filtered::<Entity, With<Button>>()
        .iter(world)
        .collect::<Vec<_>>();
    assert_eq!(buttons.len(), 1);
    let button = buttons[0];
    assert_eq!(
        world.get::<MapRoadNode>(button).unwrap().0,
        RoadId::MarketRoad
    );
    assert_eq!(
        world.get::<TravelButtonAction>(button).unwrap().0,
        RoadId::MarketRoad
    );

    let mut connections = world
        .query::<&MapConnection>()
        .iter(world)
        .map(|connection| (connection.from, connection.to))
        .collect::<Vec<_>>();
    connections.sort_unstable();
    assert_eq!(
        connections,
        [
            (RoadId::ExampleRoad, RoadId::MarketRoad),
            (RoadId::MarketRoad, RoadId::Court),
        ]
    );
}

#[test]
fn empty_unlock_map_still_displays_the_full_network_and_can_end_the_day() {
    let mut app = crate::test_support::scene_app();
    app.init_asset::<Font>()
    .insert_resource(crate::game::calendar::GameCalendar {
        total_days: 1,
        elapsed_days: 0,
    })
    .init_resource::<crate::level::InitialRoad>()
    .init_resource::<NextState<crate::game::CommuteState>>();
    let graph = RoadGraph {
        roads: [
            (RoadId::ExampleRoad, vec![RoadId::MarketRoad]),
            (RoadId::MarketRoad, vec![RoadId::Court]),
        ]
        .into(),
        initial_unlocks: default(),
    };
    let layout = MapLayout::from_graph(&graph);
    let world = app.world_mut();
    let root = world
        .spawn_scene(travel_ui_scene(None, vec![], &graph, &layout))
        .unwrap()
        .id();
    assert_eq!(world.query::<&MapRoadNode>().iter(world).count(), 3);
    assert_eq!(world.query::<&MapConnection>().iter(world).count(), 2);
    assert_eq!(world.query::<&TravelButtonAction>().iter(world).count(), 0);
    let button = world
        .query_filtered::<Entity, With<Button>>()
        .single(world)
        .unwrap();
    world.trigger(Activate { entity: button });
    world.flush();
    assert_eq!(
        world
            .resource::<crate::game::calendar::GameCalendar>()
            .elapsed_days,
        1
    );
    assert!(world.get_entity(root).is_err());
    assert_eq!(
        world
            .query::<&crate::level::scene::ending::InternshipEnded>()
            .iter(world)
            .count(),
        1
    );
    assert!(!world.contains_resource::<crate::level::day::DayEndRequested>());
}
