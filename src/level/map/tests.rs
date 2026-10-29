use super::*;

fn graph(roads: impl IntoIterator<Item = (RoadId, Vec<RoadId>)>) -> RoadGraph {
    RoadGraph {
        roads: roads.into_iter().collect(),
    }
}

fn assert_radius(layout: &MapLayout, road: RoadId, radius: f32) {
    assert!((layout.positions[&road].length() - radius).abs() < 0.0001);
}

#[test]
fn court_is_at_the_origin_even_for_an_empty_graph() {
    let layout = MapLayout::from_graph(&graph([]));
    assert_eq!(
        layout.positions,
        HashMap::from([(RoadId::Court, Vec2::ZERO)])
    );
}

#[test]
fn radius_uses_undirected_shortest_distance_without_changing_travel_rules() {
    let graph = graph([
        (RoadId::ExampleRoad, vec![RoadId::MarketRoad]),
        (RoadId::MarketRoad, vec![RoadId::Court]),
        (RoadId::Court, vec![]),
    ]);
    let original_roads = graph.roads.clone();
    let layout = MapLayout::from_graph(&graph);
    assert_eq!(layout.positions[&RoadId::Court], Vec2::ZERO);
    assert_radius(&layout, RoadId::MarketRoad, 1.0);
    assert_radius(&layout, RoadId::ExampleRoad, 2.0);
    assert_eq!(graph.roads, original_roads);
}

#[test]
fn roads_at_the_same_depth_are_spaced_evenly() {
    let layout = MapLayout::from_graph(&graph([(
        RoadId::Court,
        vec![RoadId::ExampleRoad, RoadId::MarketRoad],
    )]));
    assert_radius(&layout, RoadId::ExampleRoad, 1.0);
    assert_radius(&layout, RoadId::MarketRoad, 1.0);
    assert!(
        layout.positions[&RoadId::ExampleRoad]
            .abs_diff_eq(-layout.positions[&RoadId::MarketRoad], 0.0001)
    );
}

#[test]
fn layout_is_stable_across_node_edge_and_duplicate_order() {
    let first = graph([
        (RoadId::ExampleRoad, vec![RoadId::MarketRoad, RoadId::Court]),
        (RoadId::MarketRoad, vec![RoadId::Court]),
        (RoadId::Court, vec![]),
    ]);
    let reordered = graph([
        (RoadId::Court, vec![]),
        (RoadId::MarketRoad, vec![RoadId::Court]),
        (
            RoadId::ExampleRoad,
            vec![RoadId::Court, RoadId::MarketRoad, RoadId::Court],
        ),
    ]);
    assert_eq!(
        MapLayout::from_graph(&first).positions,
        MapLayout::from_graph(&reordered).positions
    );
}

#[test]
fn disconnected_and_destination_only_roads_are_kept() {
    let layout = MapLayout::from_graph(&graph([
        (RoadId::Court, vec![RoadId::ExampleRoad]),
        (RoadId::MarketRoad, vec![]),
    ]));
    assert_eq!(layout.positions.len(), 3);
    assert!(layout.positions.values().all(|position| position.is_finite()));
}
