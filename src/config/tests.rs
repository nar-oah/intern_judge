use super::*;
use crate::{
    game::{CommuteTime, GamePlugin, HasHelmet, Money, Stamina},
    level::LevelPlugin,
    player::Player,
};
use bevy::{
    asset::AssetPlugin, scene::ScenePlugin, state::app::StatesPlugin, time::TimeUpdateStrategy,
};
use std::time::{Duration, Instant};

#[derive(Resource)]
struct LoadingObserved;

fn assert_waiting_for_configs(world: &mut World) {
    assert_eq!(
        world.resource::<State<GameState>>().get(),
        &GameState::Loading
    );
    assert!(!world.contains_resource::<ConfigAssets>());
    assert!(!world.contains_resource::<GameSettings>());
    assert!(!world.contains_resource::<RoadGraph>());
    assert!(!world.contains_resource::<BuildingValues>());
    assert!(!world.contains_resource::<Money>());
    assert!(!world.contains_resource::<CommuteTime>());
    assert!(!world.contains_resource::<Stamina>());
    assert!(!world.contains_resource::<HasHelmet>());
    assert_eq!(world.query::<&Player>().iter(world).count(), 0);
    assert_eq!(world.query::<&RoadId>().iter(world).count(), 0);
}

#[test]
fn ron_configs_load_before_game_resources_and_scene() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        AssetPlugin {
            file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
            ..default()
        },
        ScenePlugin,
        ConfigPlugin,
        GamePlugin,
        LevelPlugin,
    ))
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
    .add_systems(OnEnter(GameState::Loading), |world: &mut World| {
        assert_waiting_for_configs(world);
        world.insert_resource(LoadingObserved);
    });
    assert_waiting_for_configs(app.world_mut());
    app.finish();
    app.cleanup();

    let started = Instant::now();
    loop {
        app.update();
        if app.world().resource::<State<GameState>>().get() == &GameState::Playing {
            break;
        }
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "RON configurations did not finish loading within 10 seconds"
        );
        std::thread::sleep(Duration::from_millis(1));
    }

    let world = app.world_mut();
    assert!(world.contains_resource::<LoadingObserved>());
    let settings = world.resource::<GameSettings>();
    assert_eq!(settings.commute_limit, 300.0);
    assert_eq!(settings.initial_money, 100);
    assert_eq!(settings.multiplier, 1.5);
    assert_eq!(settings.per_second, 5.0);
    assert_eq!(world.resource::<Money>().0, settings.initial_money);
    assert_eq!(world.resource::<CommuteTime>().elapsed, 0.0);
    assert_eq!(world.resource::<Stamina>().0, Stamina::MAX);
    assert!(!world.resource::<HasHelmet>().0);

    let graph = &world.resource::<RoadGraph>().roads;
    assert_eq!(graph.len(), 3);
    assert_eq!(
        graph[&RoadId::ExampleRoad],
        [RoadId::MarketRoad, RoadId::Court]
    );
    assert_eq!(graph[&RoadId::MarketRoad], [RoadId::Court]);
    assert!(graph[&RoadId::Court].is_empty());

    let buildings = world.resource::<BuildingValues>();
    assert_eq!(buildings.cat_reward, 20);
    assert_eq!(buildings.lottery.price, 10);
    assert_eq!(
        buildings
            .lottery
            .prizes
            .iter()
            .map(|prize| (prize.reward, prize.weight))
            .collect::<Vec<_>>(),
        [(50, 1), (10, 2), (0, 7)]
    );
    assert_eq!(buildings.convenience_stores.len(), 2);
    for (kind, price, recovery) in [
        (ConvenienceKind::Small, 10, 30.0),
        (ConvenienceKind::Large, 20, 60.0),
    ] {
        let store = &buildings.convenience_stores[&kind];
        assert_eq!(store.price, price);
        assert_eq!(store.stamina_recovery, recovery);
    }

    assert_eq!(world.query::<&Player>().iter(world).count(), 1);
    assert_eq!(
        world.query::<&RoadId>().single(world).unwrap(),
        &RoadId::ExampleRoad
    );

    // Runtime systems retain a snapshot instead of reading a changing asset.
    let handle = world.resource::<ConfigAssets>().settings.clone();
    world
        .resource_mut::<Assets<GameSettings>>()
        .get_mut(&handle)
        .unwrap()
        .commute_limit = 1.0;
    assert_eq!(world.resource::<GameSettings>().commute_limit, 300.0);
}
