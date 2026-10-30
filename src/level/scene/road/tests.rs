use crate::level::{
    BuildingType, RoadEffect, RoadId,
    scene::{
        effect::{ConstructionDetour, FallingObject},
        road_scene,
    },
};
use bevy::{app::TaskPoolPlugin, asset::AssetPlugin, prelude::*, scene::ScenePlugin};

fn scene_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        TaskPoolPlugin::default(),
        AssetPlugin::default(),
        ScenePlugin,
    ));
    app.finish();
    app.cleanup();
    app
}

#[test]
fn each_road_composes_only_its_requested_effects() {
    let effect_sets: [&[RoadEffect]; 4] = [
        &[],
        &[RoadEffect::FallingObject],
        &[RoadEffect::ConstructionDetour],
        &[RoadEffect::FallingObject, RoadEffect::ConstructionDetour],
    ];
    for road in [RoadId::ExampleRoad, RoadId::MarketRoad, RoadId::Court] {
        for effects in effect_sets {
            let mut app = scene_app();
            app.world_mut()
                .spawn_scene(road_scene(road, effects))
                .unwrap();
            let world = app.world_mut();

            assert_eq!(
                world
                    .query_filtered::<Entity, With<FallingObject>>()
                    .iter(world)
                    .count(),
                usize::from(effects.contains(&RoadEffect::FallingObject)),
                "{road:?}, {effects:?}",
            );
            assert_eq!(
                world
                    .query_filtered::<Entity, With<ConstructionDetour>>()
                    .iter(world)
                    .count(),
                usize::from(effects.contains(&RoadEffect::ConstructionDetour)),
                "{road:?}, {effects:?}",
            );
        }
    }
}

#[test]
fn entering_example_road_spawns_one_object_at_a_layout_candidate() {
    let mut app = scene_app();
    for _ in 0..16 {
        let world = app.world_mut();
        let root = world
            .spawn_scene(road_scene(
                RoadId::ExampleRoad,
                &[RoadEffect::FallingObject],
            ))
            .unwrap()
            .id();
        let transform = world
            .query_filtered::<&Transform, With<FallingObject>>()
            .single(world)
            .unwrap();

        assert!([-420.0, -250.0, 90.0, 250.0].contains(&transform.translation.x));
        assert_eq!(transform.translation.y, 320.0);
        world.despawn(root);
    }
}

#[test]
fn market_detour_stands_on_ground_between_the_stores() {
    let mut app = scene_app();
    let world = app.world_mut();
    world
        .spawn_scene(road_scene(
            RoadId::MarketRoad,
            &[RoadEffect::ConstructionDetour],
        ))
        .unwrap();
    let (transform, sprite) = world
        .query_filtered::<(&Transform, &Sprite), With<ConstructionDetour>>()
        .single(world)
        .unwrap();
    let size = sprite.custom_size.unwrap();
    let position = transform.translation.truncate();

    assert_eq!(size, Vec2::splat(60.0));
    assert_eq!(position.y - size.y / 2.0, -162.5);
    for (building, transform, sprite) in world
        .query::<(&BuildingType, &Transform, &Sprite)>()
        .iter(world)
    {
        let store_half_width = sprite.custom_size.unwrap().x / 2.0;
        match building {
            BuildingType::ToolStore => {
                assert!(transform.translation.x + store_half_width < position.x - size.x / 2.0);
            }
            BuildingType::ConvenienceStore => {
                assert!(position.x + size.x / 2.0 < transform.translation.x - store_half_width);
            }
            _ => panic!("unexpected MarketRoad building: {building:?}"),
        }
    }
}
