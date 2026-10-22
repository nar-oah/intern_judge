mod interaction;
mod navigation;
mod scene;
mod ui;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod travel_tests;
use bevy::prelude::*;

pub(crate) use navigation::TravelUi;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RoadId(pub &'static str);

const EXAMPLE_ROAD: RoadId = RoadId("example_road");
const NEXT_ROAD: RoadId = RoadId("market_road");
const COURT: RoadId = RoadId("court");
// Both roads begin at x = -500; leave room for the player's collider at the left edge.
pub(crate) const ROAD_SPAWN: Vec2 = Vec2::new(-460.0, -130.0);

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
    pub destinations: &'static [RoadId],
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
        app.init_resource::<TravelUi>()
            .add_systems(Startup, scene::example_road_scene.spawn())
            .add_systems(Update, (ui::close_on_escape, ui::sync_ui).chain())
            .add_systems(PostUpdate, interaction::interact);
    }
}
