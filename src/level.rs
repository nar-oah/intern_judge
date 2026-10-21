mod interaction;
mod scene;
#[cfg(test)]
mod tests;

use bevy::prelude::*;

/// Authored identity of a road root, independent of its runtime entity ID.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RoadId(pub &'static str);

/// Logical dimensions, kept separate from rendering for future tiled/nine-slice art.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Platform {
    pub size: Vec2,
}

/// Placeholder categories for the example road.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BuildingType {
    #[default]
    Residential,
    Shop,
    Factory,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SwitchPoint {
    pub id: &'static str,
}

#[derive(Component, Clone, Copy, Default)]
struct Interactable;

/// A player's request to interact with the overlapping target `entity`.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct InteractRequested {
    pub entity: Entity,
    pub player: Entity,
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, scene::example_road_scene.spawn())
            // Physics updates CollidingEntities during the fixed loop before PostUpdate.
            .add_systems(PostUpdate, interaction::interact);
    }
}
