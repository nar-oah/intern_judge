use crate::{
    level::{
        CurrentCase, InitialRoad, RoadEffect, RoadEffects, RoadId, SwitchPoint, TodayRoads,
        day::start_day,
        navigation::open_destinations,
        scene,
        tests::support::{activate, assert_road, navigation_app},
    },
    player::Player,
    test_support::{assert_removed, finish_app, level_app, named, scene_entities, single},
};
use bevy::{
    app::{HierarchyPropagatePlugin, PropagateSet},
    camera::{ComputedCameraValues, RenderTargetInfo, Viewport},
    prelude::*,
    ui::{ui_layout_system, ui_surface::UiSurface, update::propagate_ui_target_cameras},
};

fn assert_no_case_hud(app: &mut App) {
    let world = app.world_mut();
    assert!(
        world
            .query::<&Name>()
            .iter(world)
            .all(|name| name.as_str() != "CaseHud")
    );
    assert!(
        world
            .query::<&Text>()
            .iter(world)
            .all(|text| !text.0.starts_with("今日涉案路段："))
    );
}

fn enter_court(app: &mut App) -> Entity {
    let player = single::<Player>(app);
    let point = single::<SwitchPoint>(app);
    open_destinations(app.world_mut(), point, player);
    assert_no_case_hud(app);
    assert!(app.world().resource::<CurrentCase>().0.is_none());
    activate(app, RoadId::Court);
    assert_road(app, RoadId::Court).0
}

fn assert_case_hud(app: &mut App, court: Entity, case_road: RoadId) -> Vec<Entity> {
    assert_eq!(
        app.world()
            .resource::<CurrentCase>()
            .0
            .as_ref()
            .unwrap()
            .road,
        case_road
    );
    let world = app.world_mut();
    let panels = world
        .query::<(Entity, &Name)>()
        .iter(world)
        .filter_map(|(entity, name)| (name.as_str() == "CaseHud").then_some(entity))
        .collect::<Vec<_>>();
    assert_eq!(panels.len(), 1);
    let panel = panels[0];
    assert!(world.get::<Node>(court).is_none());
    assert!(world.get::<Node>(panel).is_some());
    // Keep the HUD in the court's UI subtree without fixing its nesting depth.
    let mut hud_root = panel;
    while let Some(parent) = world.get::<ChildOf>(hud_root) {
        if parent.parent() == court {
            break;
        }
        hud_root = parent.parent();
    }
    assert_eq!(world.get::<ChildOf>(hud_root).unwrap().parent(), court);
    assert!(world.get::<Node>(hud_root).is_some());
    let labels = scene_entities(world, panel)
        .into_iter()
        .filter_map(|entity| world.get::<Text>(entity).map(|text| text.0.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(labels, [format!("今日涉案路段：{case_road:?}")]);
    scene_entities(world, hud_root)
}

#[test]
fn court_case_hud_shows_the_selected_road_and_is_removed_when_the_scene_is_replaced() {
    for case_road in [RoadId::ExampleRoad, RoadId::MarketRoad] {
        let mut app = navigation_app();
        assert_no_case_hud(&mut app);
        if case_road == RoadId::MarketRoad {
            let player = single::<Player>(&mut app);
            let point = single::<SwitchPoint>(&mut app);
            open_destinations(app.world_mut(), point, player);
            assert_no_case_hud(&mut app);
            activate(&mut app, RoadId::MarketRoad);
            assert_road(&mut app, RoadId::MarketRoad);
            assert_no_case_hud(&mut app);
            // Exclude ExampleRoad so selection must use the other visited road.
            app.world_mut()
                .resource_mut::<RoadEffects>()
                .0
                .insert(RoadId::ExampleRoad, vec![RoadEffect::FallingObject]);
        }
        let court = enter_court(&mut app);
        let hud = assert_case_hud(&mut app, court, case_road);

        let next = scene::enter_road(app.world_mut(), RoadId::ExampleRoad);
        scene::replace_scene(app.world_mut(), [court], next);

        assert_removed(&app, &hud);
        assert_road(&mut app, RoadId::ExampleRoad);
        assert_no_case_hud(&mut app);
    }
}

#[test]
fn court_case_hud_is_laid_out_at_the_top_center_without_moving_world_entities() {
    let mut app = level_app();
    app.add_plugins((
        HierarchyPropagatePlugin::<ComputedUiTargetCamera>::new(PostUpdate),
        HierarchyPropagatePlugin::<ComputedUiRenderTargetInfo>::new(PostUpdate),
    ))
    .init_resource::<UiScale>()
    .init_resource::<UiSurface>()
    .init_resource::<bevy::text::FontCx>()
    .init_resource::<RemSize>()
    .init_resource::<RoadEffects>()
    .init_resource::<CurrentCase>()
    .insert_resource(TodayRoads([RoadId::ExampleRoad].into()))
    .add_systems(
        PostUpdate,
        (propagate_ui_target_cameras, ui_layout_system).chain(),
    )
    .configure_sets(
        PostUpdate,
        (
            PropagateSet::<ComputedUiTargetCamera>::default(),
            PropagateSet::<ComputedUiRenderTargetInfo>::default(),
        )
            .after(propagate_ui_target_cameras)
            .before(ui_layout_system),
    );
    let viewport_size = UVec2::new(1000, 600);
    app.world_mut().spawn((
        Camera2d,
        Camera {
            computed: ComputedCameraValues {
                target_info: Some(RenderTargetInfo {
                    physical_size: viewport_size,
                    scale_factor: 1.0,
                }),
                ..default()
            },
            viewport: Some(Viewport {
                physical_size: viewport_size,
                ..default()
            }),
            ..default()
        },
    ));
    finish_app(&mut app);
    let next = scene::enter_road(app.world_mut(), RoadId::Court);
    let court = app.world_mut().spawn_scene(next).unwrap().id();
    let player = single::<Player>(&mut app);
    assert_case_hud(&mut app, court, RoadId::ExampleRoad);
    let hud = named(&mut app, "CaseHud");
    let court_transform = *app.world().get::<Transform>(court).unwrap();
    let player_transform = *app.world().get::<Transform>(player).unwrap();

    app.update();

    let world = app.world();
    let size = world.get::<ComputedNode>(hud).unwrap().size();
    let center = world.get::<UiGlobalTransform>(hud).unwrap().translation;
    assert!(size.x > 0.0 && size.y > 0.0);
    assert!((center.x - viewport_size.x as f32 / 2.0).abs() <= 1.0);
    assert!(center.y - size.y / 2.0 >= 0.0);
    assert!(center.y + size.y / 2.0 < viewport_size.y as f32 / 4.0);
    assert_eq!(world.get::<Transform>(court), Some(&court_transform));
    assert_eq!(world.get::<Transform>(player), Some(&player_transform));
}

#[test]
fn starting_a_day_removes_the_previous_courts_hud() {
    let mut app = navigation_app();
    app.init_resource::<InitialRoad>();
    let court = enter_court(&mut app);
    let hud = assert_case_hud(&mut app, court, RoadId::ExampleRoad);

    start_day(app.world_mut());

    assert_removed(&app, &hud);
    assert_road(&mut app, RoadId::ExampleRoad);
    assert!(app.world().resource::<CurrentCase>().0.is_none());
    assert_no_case_hud(&mut app);
}
