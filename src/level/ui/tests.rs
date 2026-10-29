use super::*;
use bevy::{
    app::{HierarchyPropagatePlugin, PropagateSet},
    camera::{ComputedCameraValues, RenderTargetInfo, Viewport},
    text::{FontCx, RemSize},
    ui::{ui_layout_system, ui_surface::UiSurface, update::propagate_ui_target_cameras},
};

#[test]
fn native_map_geometry_and_picking_follow_viewport_resizes() {
    let mut app = crate::test_support::headless_app(Vec2::ZERO);
    app.add_plugins((
        HierarchyPropagatePlugin::<ComputedUiTargetCamera>::new(PostUpdate),
        HierarchyPropagatePlugin::<ComputedUiRenderTargetInfo>::new(PostUpdate),
    ))
    .init_resource::<UiScale>()
    .init_resource::<UiSurface>()
    .init_resource::<FontCx>()
    .init_resource::<RemSize>()
    .configure_sets(
        PostUpdate,
        (
            PropagateSet::<ComputedUiTargetCamera>::default(),
            PropagateSet::<ComputedUiRenderTargetInfo>::default(),
        )
            .after(propagate_ui_target_cameras)
            .before(ui_layout_system),
    )
    .add_systems(
        PostUpdate,
        (propagate_ui_target_cameras, ui_layout_system).chain(),
    );
    let camera = app.world_mut().spawn((Camera2d, IsDefaultUiCamera)).id();
    let graph = app.world().resource::<RoadGraph>().clone();
    let layout = MapLayout::from_graph(&graph);
    let destinations = vec![RoadId::MarketRoad, RoadId::Court];
    app.world_mut()
        .spawn_scene(travel_ui_scene(
            RoadId::ExampleRoad,
            destinations.clone(),
            &graph,
            &layout,
        ))
        .unwrap();
    app.finish();
    app.cleanup();

    for size in [
        UVec2::new(1280, 720),
        UVec2::new(360, 640),
        UVec2::new(240, 180),
    ] {
        *app.world_mut().get_mut::<Camera>(camera).unwrap() = Camera {
            computed: ComputedCameraValues {
                target_info: Some(RenderTargetInfo {
                    physical_size: size,
                    scale_factor: 1.0,
                }),
                ..default()
            },
            viewport: Some(Viewport {
                physical_size: size,
                ..default()
            }),
            ..default()
        };
        app.update();
        let world = app.world_mut();
        let mut roads = 0;
        let mut lines = 0;
        for (node, transform, picking, road, parent) in world
            .query::<(
                &ComputedNode,
                &UiGlobalTransform,
                &Pickable,
                Option<&MapRoadNode>,
                Option<&ChildOf>,
            )>()
            .iter(world)
        {
            if let Some(road) = road {
                roads += 1;
                assert_eq!(picking.is_hoverable, destinations.contains(&road.0));
                assert!(node.contains_point(*transform, transform.translation));
            } else if parent
                .is_some_and(|parent| world.get::<MapConnection>(parent.parent()).is_some())
            {
                lines += 1;
                assert!(!picking.is_hoverable);
            } else {
                continue;
            }
            assert!(node.size().min_element() > 0.0);
            for sign in [
                Vec2::new(-1.0, -1.0),
                Vec2::new(-1.0, 1.0),
                Vec2::new(1.0, -1.0),
                Vec2::ONE,
            ] {
                let corner = transform.transform_point2(node.size() * sign / 2.0);
                assert!(
                    corner.cmpge(Vec2::ZERO).all() && corner.cmple(size.as_vec2()).all(),
                    "{corner:?} outside {size:?}"
                );
            }
        }
        assert_eq!(roads, layout.positions.len());
        assert_eq!(lines, 9);
    }
}
