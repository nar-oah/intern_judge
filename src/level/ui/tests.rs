use super::*;

#[test]
fn map_displays_the_graph_but_only_outgoing_destinations_are_buttons() {
    let mut app = crate::test_support::scene_app();
    let graph = RoadGraph {
        roads: [
            (RoadId::Start, vec![RoadId::ExampleRoad]),
            (RoadId::ExampleRoad, vec![RoadId::MarketRoad]),
            (RoadId::MarketRoad, vec![RoadId::Court]),
        ]
        .into(),
    };
    let layout = MapLayout::from_graph(&graph);
    let world = app.world_mut();
    world
        .spawn_scene(travel_ui_scene(
            RoadId::ExampleRoad,
            navigation::destinations(&graph, RoadId::ExampleRoad),
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
        [
            RoadId::Start,
            RoadId::ExampleRoad,
            RoadId::MarketRoad,
            RoadId::Court,
        ]
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
            (RoadId::Start, RoadId::ExampleRoad),
            (RoadId::ExampleRoad, RoadId::MarketRoad),
            (RoadId::MarketRoad, RoadId::Court),
        ]
    );
}
