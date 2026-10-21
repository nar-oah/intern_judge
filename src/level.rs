mod interaction;
mod scene;
#[cfg(test)]
mod tests;
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RoadId(pub &'static str);

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Platform {
    pub size: Vec2,
}

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

#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct InteractRequested {
    pub entity: Entity,
    pub player: Entity,
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, scene::example_road_scene.spawn())
            .add_systems(PostUpdate, interaction::interact);
    }
}
