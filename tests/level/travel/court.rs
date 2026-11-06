use crate::{
    config::{CaseDefs, GameState},
    game::{CommuteState, GamePlugin},
    level::{
        CurrentCase, InitialRoad, RoadEffect, RoadEffects, RoadId, SceneRoot, SelectedCase,
        SwitchPoint, TodayRoads,
        day::start_day,
        scene,
        tests::support::{activate, assert_road, navigation_app, open_destinations},
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

fn assert_case_hud(app: &mut App, case_road: Option<RoadId>) -> Vec<Entity> {
    let root = named(app, "CaseHudRoot");
    let panel = named(app, "CaseHud");
    let world = app.world();
    assert!(world.get::<SceneRoot>(root).is_none());
    assert!(world.get::<ChildOf>(root).is_none());
    assert!(world.get::<Node>(root).is_some());
    assert_eq!(world.get::<ChildOf>(panel).unwrap().parent(), root);
    assert_eq!(
        world.get::<Visibility>(root),
        Some(&if case_road.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        })
    );
    let entities = scene_entities(world, root);
    let labels = entities
        .iter()
        .filter_map(|&entity| world.get::<Text>(entity).map(|text| text.0.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        labels,
        [case_road.map_or_else(String::new, |road| format!("今日涉案路段：{road:?}"))]
    );
    entities
}

fn enter_court(app: &mut App) -> Entity {
    let player = single::<Player>(app);
    let point = single::<SwitchPoint>(app);
    open_destinations(app.world_mut(), point, player);
    assert_case_hud(app, None);
    assert!(app.world().resource::<CurrentCase>().0.is_none());
    activate(app, RoadId::Court);
    app.update();
    assert_road(app, RoadId::Court).0
}

#[test]
fn case_hud_shows_the_selected_road_and_survives_road_and_map_replacement() {
    for case_road in [RoadId::ExampleRoad, RoadId::MarketRoad] {
        let mut app = navigation_app();
        let hud = assert_case_hud(&mut app, None);
        if case_road == RoadId::MarketRoad {
            let player = single::<Player>(&mut app);
            let point = single::<SwitchPoint>(&mut app);
            open_destinations(app.world_mut(), point, player);
            assert_eq!(assert_case_hud(&mut app, None), hud);
            activate(&mut app, RoadId::MarketRoad);
            assert_road(&mut app, RoadId::MarketRoad);
            assert_eq!(assert_case_hud(&mut app, None), hud);
            // Exclude ExampleRoad so selection must use the other visited road.
            app.world_mut()
                .resource_mut::<RoadEffects>()
                .0
                .insert(RoadId::ExampleRoad, vec![RoadEffect::FallingObject]);
        }
        let court = enter_court(&mut app);
        assert_eq!(assert_case_hud(&mut app, Some(case_road)), hud);

        app.world_mut()
            .trigger(scene::EnterRoad(RoadId::ExampleRoad));
        app.world_mut().flush();
        app.update();

        assert!(app.world().get_entity(court).is_err());
        assert_road(&mut app, RoadId::ExampleRoad);
        assert_eq!(assert_case_hud(&mut app, Some(case_road)), hud);

        let player = single::<Player>(&mut app);
        let point = single::<SwitchPoint>(&mut app);
        open_destinations(app.world_mut(), point, player);
        app.update();
        assert_eq!(assert_case_hud(&mut app, Some(case_road)), hud);
    }
}

#[test]
fn case_hud_refreshes_from_current_case_while_paused_and_hides_when_it_is_cleared() {
    let mut app = navigation_app();
    let hud = assert_case_hud(&mut app, None);
    app.world_mut()
        .resource_mut::<NextState<CommuteState>>()
        .set(CommuteState::Paused);
    let case = app.world().resource::<CaseDefs>().0[0].clone();
    for road in [RoadId::MarketRoad, RoadId::ExampleRoad] {
        app.world_mut().resource_mut::<CurrentCase>().0 = Some(SelectedCase {
            road,
            case: case.clone(),
        });
        app.update();
        assert_eq!(assert_case_hud(&mut app, Some(road)), hud);
    }

    app.world_mut().resource_mut::<CurrentCase>().0 = None;
    app.update();

    assert_eq!(assert_case_hud(&mut app, None), hud);
}

#[test]
fn case_hud_is_laid_out_at_the_top_center_without_moving_world_entities() {
    let mut app = level_app();
    app.add_plugins(GamePlugin);
    app.add_observer(scene::enter_road)
        .add_plugins((
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
    app.world_mut().trigger(scene::EnterRoad(RoadId::Court));
    app.world_mut().flush();
    let court = single::<SceneRoot>(&mut app);
    let player = single::<Player>(&mut app);
    let hud = named(&mut app, "CaseHud");
    let court_transform = *app.world().get::<Transform>(court).unwrap();
    let player_transform = *app.world().get::<Transform>(player).unwrap();

    app.update();

    assert_case_hud(&mut app, Some(RoadId::ExampleRoad));
    let world = app.world();
    assert!(world.get::<Node>(court).is_none());
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
fn starting_a_day_hides_the_existing_case_hud() {
    let mut app = navigation_app();
    app.init_resource::<InitialRoad>();
    enter_court(&mut app);
    let hud = assert_case_hud(&mut app, Some(RoadId::ExampleRoad));

    start_day(app.world_mut());
    app.update();

    assert_road(&mut app, RoadId::ExampleRoad);
    assert!(app.world().resource::<CurrentCase>().0.is_none());
    assert_eq!(assert_case_hud(&mut app, None), hud);
}

#[test]
fn leaving_playing_despawns_the_case_hud_and_reentering_recreates_it() {
    let mut app = navigation_app();
    enter_court(&mut app);
    let hud = assert_case_hud(&mut app, Some(RoadId::ExampleRoad));

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::MainMenu);
    app.update();

    assert_removed(&app, &hud);

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    let next_hud = assert_case_hud(&mut app, Some(RoadId::ExampleRoad));
    assert!(next_hud.iter().all(|entity| !hud.contains(entity)));
}
