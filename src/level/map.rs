#[cfg(test)]
mod tests;
use super::RoadId;
use crate::config::RoadGraph;
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

#[derive(Resource, Debug)]
pub(super) struct MapLayout {
    pub(super) positions: HashMap<RoadId, Vec2>,
}

impl MapLayout {
    pub(super) fn from_graph(graph: &RoadGraph) -> Self {
        let mut neighbors = HashMap::<RoadId, Vec<RoadId>>::new();
        neighbors.entry(RoadId::Court).or_default();
        for (&road, destinations) in &graph.roads {
            neighbors.entry(road).or_default().extend(destinations);
            for &destination in destinations {
                neighbors.entry(destination).or_default().push(road);
            }
        }

        let mut depths = HashMap::from([(RoadId::Court, 0)]);
        let mut pending = VecDeque::from([RoadId::Court]);
        while let Some(road) = pending.pop_front() {
            let next_depth = depths[&road] + 1;
            for &neighbor in &neighbors[&road] {
                if let std::collections::hash_map::Entry::Vacant(entry) = depths.entry(neighbor) {
                    entry.insert(next_depth);
                    pending.push_back(neighbor);
                }
            }
        }

        let mut rings = vec![Vec::new(); neighbors.len()];
        for &road in neighbors.keys() {
            if road != RoadId::Court {
                let depth = depths.get(&road).copied().unwrap_or(1);
                rings[depth].push(road);
            }
        }

        let mut positions = HashMap::from([(RoadId::Court, Vec2::ZERO)]);
        for (depth, mut roads) in rings.into_iter().enumerate() {
            roads.sort_unstable();
            for (index, road) in roads.iter().copied().enumerate() {
                let angle = std::f32::consts::TAU * index as f32 / roads.len() as f32
                    - std::f32::consts::FRAC_PI_2;
                positions.insert(road, Vec2::from_angle(angle) * depth as f32);
            }
        }
        Self { positions }
    }
}
