use crate::{
    config::{BuildingValues, GameSettings, GameState, RoadGraph},
    game::Stamina,
};
use avian2d::prelude::*;
use bevy::{
    asset::AssetPlugin, input::InputPlugin, prelude::*, scene::ScenePlugin,
    state::app::StatesPlugin, time::TimeUpdateStrategy,
};
use std::time::Duration;

const FIXED_TIMESTEP: Duration = Duration::from_micros(15_625);

/// Build a headless app without spawning game entities.
pub(crate) fn headless_app(gravity: Vec2) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        TransformPlugin,
        AssetPlugin::default(),
        ScenePlugin,
        InputPlugin,
        PhysicsPlugins::default().with_length_unit(20.0),
    ))
    .insert_resource(Gravity(gravity))
    .init_resource::<Stamina>()
    .insert_resource(Time::<Fixed>::from_duration(FIXED_TIMESTEP))
    .insert_resource(TimeUpdateStrategy::ManualDuration(FIXED_TIMESTEP));
    insert_config(&mut app);
    app
}

pub(crate) fn insert_config(app: &mut App) {
    app.insert_state(GameState::Playing)
        .insert_resource(
            ron::from_str::<GameSettings>(include_str!("../assets/game.settings.ron")).unwrap(),
        )
        .insert_resource(
            ron::from_str::<RoadGraph>(include_str!("../assets/roads.graph.ron")).unwrap(),
        )
        .insert_resource(
            ron::from_str::<BuildingValues>(include_str!("../assets/buildings.values.ron"))
                .unwrap(),
        );
}

/// Initialize plugins and run startup; later updates advance one fixed timestep.
pub(crate) fn finish_app(app: &mut App) {
    app.finish();
    app.cleanup();
    app.update();
}

pub(crate) fn advance(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}
