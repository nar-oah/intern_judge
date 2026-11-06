use super::*;
use crate::game::{CommuteTime, GamePlugin, HasHelmet, Money, Stamina, calendar::GameCalendar};
use bevy::{asset::AssetPlugin, scene::ScenePlugin, state::app::StatesPlugin};
use bevy_replique::prelude::{RepliqueDialogue, RepliquePlugin};
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

#[test]
fn shipped_ron_assets_load_into_main_menu_before_gameplay_initializes() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        AssetPlugin {
            file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
            ..default()
        },
        ScenePlugin,
        RepliquePlugin,
        ConfigPlugin,
        GamePlugin,
    ));
    app.init_asset::<Font>()
        .init_asset_loader::<bevy::text::FontLoader>();
    let court = ["verdict", "prosecution", "defense"].map(|name| {
        app.world()
            .resource::<AssetServer>()
            .load::<RepliqueDialogue>(format!("dialogue/{name}.rep"))
    });
    assert_eq!(
        app.world().resource::<State<GameState>>().get(),
        &GameState::Loading
    );
    assert!(!app.world().contains_resource::<GameSettings>());
    assert!(!app.world().contains_resource::<CaseDefs>());
    assert!(!app.world().contains_resource::<Money>());
    assert!(!app.world().contains_resource::<GameCalendar>());
    app.finish();
    app.cleanup();

    let deadline = Instant::now() + Duration::from_secs(10);
    while app.world().resource::<State<GameState>>().get() != &GameState::MainMenu
        || court.iter().any(|handle| {
            app.world()
                .resource::<Assets<RepliqueDialogue>>()
                .get(handle)
                .is_none()
        })
    {
        app.update();
        assert!(
            Instant::now() < deadline,
            "configuration and court dialogue assets did not finish loading"
        );
        std::thread::sleep(Duration::from_millis(1));
    }

    // Load the shipped assets so configuration and dialogue stay compatible.
    let world = app.world();
    assert_eq!(world.resource::<GameSettings>().total_days, 28);
    assert_eq!(
        world.resource::<RoadGraph>().initial_unlocks,
        HashMap::from([
            (RoadId::ExampleRoad, true),
            (RoadId::MarketRoad, false),
            (RoadId::Court, true),
            (RoadId::EmptyRoad1, false),
            (RoadId::EmptyRoad2, false),
            (RoadId::EmptyRoad3, false),
            (RoadId::EmptyRoad4, false),
            (RoadId::EmptyRoad5, false),
            (RoadId::EmptyRoad6, false),
            (RoadId::EmptyRoad7, false),
        ])
    );
    let graph = world.resource::<RoadGraph>();
    for road in [
        RoadId::EmptyRoad1,
        RoadId::EmptyRoad2,
        RoadId::EmptyRoad3,
        RoadId::EmptyRoad4,
        RoadId::EmptyRoad5,
        RoadId::EmptyRoad6,
        RoadId::EmptyRoad7,
    ] {
        assert!(graph.roads[&RoadId::MarketRoad].contains(&road));
        assert_eq!(graph.roads[&road], [RoadId::MarketRoad]);
    }
    world.resource::<BuildingValues>();
    let cases = &world.resource::<CaseDefs>().0;
    assert_eq!(cases.len(), 6);
    let effects = [
        RoadEffect::FallingObject,
        RoadEffect::ConstructionDetour,
        RoadEffect::PlaceholderA,
        RoadEffect::PlaceholderB,
    ];
    for (index, &first) in effects.iter().enumerate() {
        for &second in &effects[index + 1..] {
            let count = cases
                .iter()
                .filter(|case| {
                    (case.guilty == first && case.not_guilty == second)
                        || (case.guilty == second && case.not_guilty == first)
                })
                .count();
            assert_eq!(count, 1, "{first:?} / {second:?}");
        }
    }
    let nodes = cases
        .iter()
        .map(|case| &case.dialogue_node)
        .collect::<HashSet<_>>();
    assert_eq!(nodes.len(), cases.len());
    let dialogues = world.resource::<Assets<RepliqueDialogue>>();
    assert!(
        dialogues
            .get(&court[0])
            .unwrap()
            .dialogue()
            .get_node(&"verdict".into())
            .is_some()
    );
    for case in cases {
        assert_ne!(case.guilty, case.not_guilty);
        for handle in &court[1..] {
            assert!(
                dialogues
                    .get(handle)
                    .unwrap()
                    .dialogue()
                    .get_node(&case.dialogue_node.as_str().into())
                    .is_some(),
                "{}",
                case.dialogue_node,
            );
        }
    }
    assert!(!world.contains_resource::<Money>());
    assert!(!world.contains_resource::<CommuteTime>());
    assert!(!world.contains_resource::<Stamina>());
    assert!(!world.contains_resource::<HasHelmet>());
    assert!(!world.contains_resource::<GameCalendar>());

    for _ in 0..3 {
        app.update();
    }
    assert_eq!(
        app.world().resource::<State<GameState>>().get(),
        &GameState::MainMenu
    );
    assert!(!app.world().contains_resource::<Money>());

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();
    let world = app.world();
    assert_eq!(
        world.resource::<Money>().0,
        world.resource::<GameSettings>().initial_money
    );
    assert_eq!(world.resource::<GameCalendar>().total_days, 28);
    assert_eq!(world.resource::<GameCalendar>().elapsed_days, 0);
}
