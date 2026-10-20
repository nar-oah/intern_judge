mod interaction;
mod scene;

use bevy::prelude::*;

/// Authored identity of a road root, independent of its runtime entity ID.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RoadId(pub &'static str);

/// Logical dimensions, kept separate from rendering for future tiled/nine-slice art.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Platform {
    pub size: Vec2,
}

// Placeholder categories until the design document is available in the repository.
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
struct Interactable {
    radius: f32,
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, scene::example_road_scene.spawn())
            .add_systems(
                PostUpdate,
                interaction::interact.after(TransformSystems::Propagate),
            );
    }
}
