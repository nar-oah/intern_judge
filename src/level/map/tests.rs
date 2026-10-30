use super::*;

fn graph(roads: impl IntoIterator<Item = (RoadId, Vec<RoadId>)>) -> RoadGraph {
    RoadGraph {
        roads: roads.into_iter().collect(),
    }
}

#[test]
fn court_is_centered_and_roads_follow_the_shortest_undirected_distance() {
    // The reverse Court -> MarketRoad edge and the cycle must not affect depth.
    let layout = MapLayout::from_graph(&graph([
        (RoadId::ExampleRoad, vec![RoadId::MarketRoad]),
        (RoadId::Court, vec![RoadId::MarketRoad]),
        (RoadId::MarketRoad, vec![RoadId::Court, RoadId::ExampleRoad]),
    ]));
    assert_eq!(layout.positions[&RoadId::Court], Vec2::ZERO);
    for (road, radius) in [(RoadId::MarketRoad, 1.0), (RoadId::ExampleRoad, 2.0)] {
        assert!((layout.positions[&road].length() - radius).abs() < 0.0001);
    }
}

#[test]
fn same_depth_roads_are_separated_and_layout_is_stable_when_edges_are_reordered() {
    let first = graph([
        (RoadId::ExampleRoad, vec![RoadId::MarketRoad, RoadId::Court]),
        (RoadId::MarketRoad, vec![RoadId::Court]),
    ]);
    let reordered = graph([
        (RoadId::MarketRoad, vec![RoadId::Court]),
        (
            RoadId::ExampleRoad,
            vec![RoadId::Court, RoadId::MarketRoad, RoadId::Court],
        ),
    ]);
    let layout = MapLayout::from_graph(&first);
    assert_eq!(
        layout.positions,
        MapLayout::from_graph(&reordered).positions
    );
    assert!(
        layout.positions[&RoadId::ExampleRoad]
            .abs_diff_eq(-layout.positions[&RoadId::MarketRoad], 0.0001)
    );
    assert!((layout.positions[&RoadId::ExampleRoad].length() - 1.0).abs() < 0.0001);
}
