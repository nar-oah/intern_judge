use super::*;

#[test]
fn map_displays_the_graph_but_only_outgoing_destinations_are_buttons() {
    let mut app = crate::test_support::scene_app();
    let graph = RoadGraph {
        roads: [
            (RoadId::Start, vec![RoadId::ExampleRoad]),
            (RoadId::ExampleRoad, vec![RoadId::MarketRoad]),
            (RoadId::MarketRoad, vec![RoadId::Court]),
        ].into(),
    };
    let layout = MapLayout::from_graph(&graph);
    let world = app.world_mut();
    world.spawn_scene(travel_ui_scene(
        RoadId::ExampleRoad,
        navigation::destinations(&graph, RoadId::ExampleRoad),
        &graph,
        &layout,
    )).unwrap();

    let mut displayed = world.query::<&MapRoadNode>().iter(world)
        .map(|node| node.0).collect::<Vec<_>>();
    displayed.sort_unstable();
    assert_eq!(displayed, [
        RoadId::Start, RoadId::ExampleRoad, RoadId::MarketRoad, RoadId::Court,
    ]);

    let actions = world.query_filtered::<&TravelButtonAction, With<Button>>()
        .iter(world).map(|action| action.0).collect::<Vec<_>>();
    assert_eq!(actions, [RoadId::MarketRoad]);
}
