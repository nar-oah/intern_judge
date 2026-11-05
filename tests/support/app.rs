use super::insert_config;
use crate::game::{CommuteState, Stamina};
use avian2d::prelude::*;
use bevy::{
    asset::AssetPlugin, input::InputPlugin, prelude::*, scene::ScenePlugin,
    state::app::StatesPlugin, time::TimeUpdateStrategy,
};
use std::time::Duration;

const FIXED_TIMESTEP: Duration = Duration::from_micros(15_625);

/// Scene spawning needs assets, but does not need rendering or physics.
pub(crate) fn scene_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), ScenePlugin));
    app.init_asset::<Font>()
        .init_asset_loader::<bevy::text::FontLoader>();
    app
}

/// Scene, state and keyboard support without physics or game entities.
pub(crate) fn level_app() -> App {
    let mut app = scene_app();
    app.add_plugins((StatesPlugin, InputPlugin))
        .init_state::<CommuteState>();
    insert_config(&mut app);
    app
}

/// Build a headless app without spawning game entities.
pub(crate) fn headless_app(gravity: Vec2) -> App {
    let mut app = scene_app();
    app.add_plugins((
        StatesPlugin,
        TransformPlugin,
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
