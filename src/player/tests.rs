use super::*;
use bevy::{
    asset::AssetPlugin, input::InputPlugin, scene::ScenePlugin, time::TimeUpdateStrategy,
};
use std::time::Duration;

fn headless_app(gravity: Vec2) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        AssetPlugin::default(),
        ScenePlugin,
        InputPlugin,
        PhysicsPlugins::default().with_length_unit(20.0),
        PlayerPlugin,
        crate::level::LevelPlugin,
    ))
    .insert_resource(Gravity(gravity))
    // Match Bevy's default fixed timestep with an exact duration so each update
    // after startup advances precisely one physics step.
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_micros(
        15_625,
    )));
    app.finish();
    app.cleanup();
    app.update();
    app
}

fn player_entity(app: &mut App) -> Entity {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<Player>>()
        .single(world)
        .expect("Startup should spawn exactly one player")
}

fn velocity(app: &App, player: Entity) -> Vec2 {
    app.world().get::<LinearVelocity>(player).unwrap().0
}

#[test]
fn startup_spawns_the_player_scene_with_input_and_physics() {
    let mut app = headless_app(Vec2::ZERO);
    let player = player_entity(&mut app);
    let entity = app.world().entity(player);

    assert_eq!(
        entity.get::<Transform>().unwrap().translation,
        Vec3::new(0.0, -80.0, 0.0)
    );
    assert_eq!(
        entity.get::<Sprite>().unwrap().custom_size,
        Some(Vec2::new(30.0, 50.0))
    );
    assert_eq!(entity.get::<RigidBody>(), Some(&RigidBody::Dynamic));
    assert!(entity.contains::<Collider>());
    assert!(entity.contains::<LockedAxes>());
    assert!(entity.contains::<Friction>());
    assert!(entity.contains::<Restitution>());
    assert!(entity.contains::<ShapeHits>());
    assert!(entity.contains::<InputMap<PlayerAction>>());
    assert!(entity.contains::<ActionState<PlayerAction>>());
    assert_eq!(entity.get::<ShapeCaster>().unwrap().max_distance, 6.0);
    assert_eq!(velocity(&app, player), Vec2::ZERO);
}

#[test]
fn keyboard_moves_left_and_right_and_opposite_inputs_cancel() {
    let mut app = headless_app(Vec2::ZERO);
    let player = player_entity(&mut app);

    KeyCode::KeyA.press(app.world_mut());
    app.update();
    assert_eq!(velocity(&app, player), Vec2::new(-220.0, 0.0));
    assert!(app.world().get::<Position>(player).unwrap().0.x < 0.0);

    KeyCode::KeyD.press(app.world_mut());
    app.update();
    assert_eq!(velocity(&app, player), Vec2::ZERO);

    KeyCode::KeyA.release(app.world_mut());
    app.update();
    assert_eq!(velocity(&app, player), Vec2::new(220.0, 0.0));

    KeyCode::KeyD.release(app.world_mut());
    app.update();
    assert_eq!(velocity(&app, player), Vec2::ZERO);
}

#[test]
fn player_lands_on_the_level_and_jumps_only_on_a_grounded_press() {
    let mut app = headless_app(Vec2::NEG_Y * 1000.0);
    let player = player_entity(&mut app);

    for _ in 0..128 {
        app.update();
    }

    // The lower platform's top is -162.5 and the player's half-height is 25.
    let resting_y = app.world().get::<Position>(player).unwrap().0.y;
    assert!((resting_y - -137.5).abs() < 2.0);
    assert!(velocity(&app, player).y.abs() < 1.0);
    assert!(
        app.world()
            .get::<ShapeHits>(player)
            .unwrap()
            .0
            .iter()
            .any(|hit| (-hit.normal2).dot(Vec2::Y) > 0.7),
        "the actual physics ground probe should see the platform"
    );

    KeyCode::Space.press(app.world_mut());
    app.update();
    assert!(velocity(&app, player).y > 350.0);
    assert!(app.world().get::<Position>(player).unwrap().0.y > resting_y);

    KeyCode::Space.release(app.world_mut());
    for _ in 0..8 {
        app.update();
    }
    assert!(app.world().get::<ShapeHits>(player).unwrap().0.is_empty());

    let airborne_velocity = velocity(&app, player).y;
    KeyCode::Space.press(app.world_mut());
    app.update();
    assert!(
        velocity(&app, player).y < airborne_velocity,
        "a new press in the air must not reset the jump velocity"
    );

    // Keep Space held through landing: it must not trigger another jump.
    for _ in 0..128 {
        app.update();
    }
    assert!((app.world().get::<Position>(player).unwrap().0.y - resting_y).abs() < 2.0);
    assert!(velocity(&app, player).y.abs() < 1.0);
}

#[test]
fn wall_and_ceiling_hits_do_not_allow_jumping() {
    let mut app = headless_app(Vec2::ZERO);
    let player = player_entity(&mut app);
    app.world_mut()
        .get_mut::<ActionState<PlayerAction>>(player)
        .unwrap()
        .press(&PlayerAction::Jump);
    app.world_mut().get_mut::<LinearVelocity>(player).unwrap().0 = Vec2::new(0.0, -100.0);

    // Isolate the controller's normal filtering from the physics simulation.
    // These represent both wall directions and a ceiling, never a floor.
    app.world_mut().get_mut::<ShapeHits>(player).unwrap().0 =
        [Vec2::X, Vec2::NEG_X, Vec2::Y]
            .into_iter()
            .map(|normal2| ShapeHitData {
                entity: Entity::PLACEHOLDER,
                distance: 0.0,
                point1: Vec2::ZERO,
                point2: Vec2::ZERO,
                normal1: -normal2,
                normal2,
            })
            .collect();
    app.world_mut().run_schedule(FixedUpdate);

    assert_eq!(velocity(&app, player).y, -100.0);
}
