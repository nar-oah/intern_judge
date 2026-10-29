use super::RoadId;
use crate::config::RoadGraph;
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

/// Map units use one unit of radius per hop from the court.
#[derive(Resource, Debug)]
pub(super) struct MapLayout {
    pub(super) positions: HashMap<RoadId, Vec2>,
}

impl FromWorld for MapLayout {
    fn from_world(world: &mut World) -> Self {
        Self::from_graph(world.resource::<RoadGraph>())
    }
}

impl MapLayout {
    pub(super) fn from_graph(graph: &RoadGraph) -> Self {
        // Ordered IDs make placement independent of hash and adjacency order.
        let mut neighbors = BTreeMap::<RoadId, BTreeSet<RoadId>>::new();
        neighbors.entry(RoadId::Court).or_default();
        for (&road, destinations) in &graph.roads {
            neighbors.entry(road).or_default();
            for &destination in destinations {
                neighbors.entry(road).or_default().insert(destination);
                neighbors.entry(destination).or_default().insert(road);
            }
        }

        // Only the layout treats roads as undirected; travel rules stay unchanged.
        let mut depths = BTreeMap::from([(RoadId::Court, 0)]);
        let mut pending = VecDeque::from([RoadId::Court]);
        while let Some(road) = pending.pop_front() {
            let next_depth = depths[&road] + 1;
            for &neighbor in &neighbors[&road] {
                if let std::collections::btree_map::Entry::Vacant(entry) = depths.entry(neighbor) {
                    entry.insert(next_depth);
                    pending.push_back(neighbor);
                }
            }
        }

        // Disconnected roads remain visible on one extra outer ring.
        let disconnected_depth = depths.values().copied().max().unwrap_or(0) + 1;
        let mut rings = BTreeMap::<usize, Vec<RoadId>>::new();
        for &road in neighbors.keys() {
            if road != RoadId::Court {
                let depth = depths.get(&road).copied().unwrap_or(disconnected_depth);
                rings.entry(depth).or_default().push(road);
            }
        }

        let mut positions = HashMap::from([(RoadId::Court, Vec2::ZERO)]);
        for (depth, roads) in rings {
            for (index, road) in roads.iter().copied().enumerate() {
                let angle = std::f32::consts::TAU * index as f32 / roads.len() as f32
                    - std::f32::consts::FRAC_PI_2;
                positions.insert(road, Vec2::from_angle(angle) * depth as f32);
            }
        }
        Self { positions }
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(layout.positions, HashMap::from([(RoadId::Court, Vec2::ZERO)]));
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
        assert!(layout.positions[&RoadId::ExampleRoad]
            .abs_diff_eq(-layout.positions[&RoadId::MarketRoad], 0.0001));
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
            (RoadId::ExampleRoad, vec![RoadId::Court, RoadId::MarketRoad, RoadId::Court]),
        ]);
        assert_eq!(MapLayout::from_graph(&first).positions,
            MapLayout::from_graph(&reordered).positions);
    }

    #[test]
    fn disconnected_and_destination_only_roads_are_kept() {
        let layout = MapLayout::from_graph(&graph([
            (RoadId::Court, vec![RoadId::ExampleRoad]),
            (RoadId::MarketRoad, vec![]),
        ]));
        assert_eq!(layout.positions.len(), 3);
        assert_radius(&layout, RoadId::ExampleRoad, 1.0);
        assert_radius(&layout, RoadId::MarketRoad, 2.0);
        assert!(layout.positions.values().all(|position| position.is_finite()));
    }
}
