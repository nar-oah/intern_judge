use super::*;
use crate::game::{GamePlugin, Money};
use bevy::{asset::AssetPlugin, state::app::StatesPlugin};
use std::time::{Duration, Instant};

#[test]
fn shipped_ron_assets_load_before_gameplay_initializes() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        AssetPlugin {
            file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
            ..default()
        },
        ConfigPlugin,
        GamePlugin,
    ));
    assert_eq!(app.world().resource::<State<GameState>>().get(), &GameState::Loading);
    assert!(!app.world().contains_resource::<GameSettings>());
    assert!(!app.world().contains_resource::<Money>());
    app.finish();
    app.cleanup();

    let deadline = Instant::now() + Duration::from_secs(10);
    while app.world().resource::<State<GameState>>().get() != &GameState::Playing {
        app.update();
        assert!(Instant::now() < deadline, "RON assets did not finish loading");
        std::thread::sleep(Duration::from_millis(1));
    }

    // Successful deserialization and transition exercise all three real asset
    // loaders without pinning balance values or the shipped road topology.
    let world = app.world();
    assert_eq!(world.resource::<Money>().0, world.resource::<GameSettings>().initial_money);
    world.resource::<RoadGraph>();
    world.resource::<BuildingValues>();
}
