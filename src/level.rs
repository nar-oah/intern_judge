mod interaction;
mod navigation;
mod scene;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod travel_tests;
mod ui;
use bevy::prelude::*;

#[derive(States, Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum GamePhase {
    #[default]
    Exploring,
    ChoosingDestination,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RoadId(pub &'static str);

const EXAMPLE_ROAD: RoadId = RoadId("example_road");
const NEXT_ROAD: RoadId = RoadId("market_road");
const COURT: RoadId = RoadId("court");
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
        app.init_state::<GamePhase>()
            .init_resource::<navigation::DestinationChoices>()
            .add_message::<ui::TravelButtonAction>()
            .add_systems(Startup, |world: &mut World| {
                scene::spawn_road(world, EXAMPLE_ROAD).unwrap();
            })
            .add_systems(
                OnEnter(GamePhase::ChoosingDestination),
                (navigation::stop_horizontal_motion, ui::spawn_ui),
            )
            .add_systems(
                OnExit(GamePhase::ChoosingDestination),
                navigation::clear_destinations,
            )
            .add_systems(
                Update,
                (ui::close_on_escape, ui::handle_buttons)
                    .chain()
                    .run_if(in_state(GamePhase::ChoosingDestination)),
            )
            .add_systems(
                PostUpdate,
                interaction::interact.run_if(in_state(GamePhase::Exploring)),
            );
    }
}
