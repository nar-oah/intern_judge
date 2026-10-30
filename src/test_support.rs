use crate::{
    config::{
        BuildingValues, ConvenienceKind, ConvenienceValues, GameSettings, GameState, LotteryPrize,
        LotteryValues, RoadGraph,
    },
    game::Stamina,
    level::RoadId,
};
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

pub(crate) fn insert_config(app: &mut App) {
    // Behavioral tests use their own data so balancing the shipped RON assets
    // does not change their expectations. Config tests exercise the real loader.
    app.insert_state(GameState::Playing)
        .insert_resource(GameSettings {
            commute_limit: 8.0,
            initial_money: 73,
            multiplier: 2.0,
            per_second: 8.0,
        })
        .insert_resource(RoadGraph {
            roads: [
                (RoadId::ExampleRoad, vec![RoadId::MarketRoad, RoadId::Court]),
                (RoadId::MarketRoad, vec![RoadId::Court]),
                (RoadId::Court, vec![]),
            ]
            .into(),
        })
        .insert_resource(BuildingValues {
            cat_reward: 17,
            lottery: LotteryValues {
                price: 7,
                prizes: vec![LotteryPrize {
                    reward: 13,
                    weight: 1,
                }],
            },
            convenience_stores: [
                (
                    ConvenienceKind::Small,
                    ConvenienceValues {
                        price: 7,
                        stamina_recovery: 25.0,
                    },
                ),
                (
                    ConvenienceKind::Large,
                    ConvenienceValues {
                        price: 13,
                        stamina_recovery: 55.0,
                    },
                ),
            ]
            .into(),
        });
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
