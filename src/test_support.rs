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
    .insert_resource(Time::<Fixed>::from_duration(FIXED_TIMESTEP))
    .insert_resource(TimeUpdateStrategy::ManualDuration(FIXED_TIMESTEP));
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
