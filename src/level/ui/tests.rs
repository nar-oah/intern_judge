use super::*;

#[test]
fn connection_is_a_single_center_to_center_line_in_viewport_units() {
    let mut world = World::new();
    let line = world
        .spawn_scene(connection_scene(
            MapConnection {
                from: RoadId::ExampleRoad,
                to: RoadId::Court,
            },
            Vec2::new(6.0, 9.0),
            Vec2::new(18.0, 25.0),
            RoadId::ExampleRoad,
            &[RoadId::Court],
        ))
        .unwrap()
        .id();

    let node = world.get::<Node>(line).unwrap();
    assert_eq!(node.left, vmin(12));
    assert_eq!(node.top, vmin(17));
    assert_eq!(node.width, vmin(20));
    assert_eq!(node.height, vmin(0.25));
    assert_eq!(node.min_height, px(1));
    let transform = world.get::<UiTransform>(line).unwrap();
    assert_eq!(transform.translation, Val2::percent(-50.0, -50.0));
    assert!((transform.rotation * Vec2::X - Vec2::new(0.6, 0.8)).length() < 0.0001);
    assert!(!world.get::<Pickable>(line).unwrap().is_hoverable);
    assert!(world.get::<Children>(line).is_none());
}
