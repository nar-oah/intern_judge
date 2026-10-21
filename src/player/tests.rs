use super::*;
use bevy::{asset::AssetPlugin, input::InputPlugin, scene::ScenePlugin, time::TimeUpdateStrategy};
use std::time::Duration;

fn game(gravity: Vec2) -> (App, Entity) {
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
    // One update after startup advances exactly one default fixed timestep.
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_micros(
        15_625,
    )));
    app.finish();
    app.cleanup();
    app.update();
    let world = app.world_mut();
    let player = world
        .query_filtered::<Entity, With<Player>>()
        .single(world)
        .expect("the player scene should spawn exactly one player");
    (app, player)
}

fn advance(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}

fn position(app: &App, player: Entity) -> Vec2 {
    app.world().get::<Position>(player).unwrap().0
}

fn velocity(app: &App, player: Entity) -> Vec2 {
    app.world().get::<LinearVelocity>(player).unwrap().0
}

fn move_away_from_level(app: &mut App, player: Entity) {
    app.world_mut().entity_mut(player).insert((
        Transform::from_xyz(1000.0, 0.0, 0.0),
        Position(Vec2::new(1000.0, 0.0)),
    ));
}

#[test]
fn player_scene_initializes_rendering_physics_and_keyboard_input() {
    let (mut app, player) = game(Vec2::ZERO);
    let entity = app.world().entity(player);
    assert_eq!(position(&app, player), Vec2::new(0.0, -80.0));
    assert_eq!(velocity(&app, player), Vec2::ZERO);
    assert_eq!(
        entity.get::<Sprite>().unwrap().custom_size,
        Some(Vec2::new(30.0, 50.0))
    );
    assert_eq!(entity.get::<RigidBody>(), Some(&RigidBody::Dynamic));
    assert!(entity.contains::<Collider>());
    assert!(entity.contains::<LockedAxes>());
    assert!(entity.contains::<Friction>());
    assert!(entity.contains::<Restitution>());
    assert!(entity.contains::<ShapeCaster>());
    assert!(entity.contains::<ShapeHits>());
    assert!(entity.contains::<InputMap<PlayerAction>>());

    KeyCode::KeyE.press(app.world_mut());
    app.update();
    assert!(
        app.world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap()
            .just_pressed(&PlayerAction::Interact)
    );
}

#[test]
fn a_and_d_move_the_player_and_cancel_when_held_together() {
    let (mut app, player) = game(Vec2::ZERO);
    for (key, pressed, expected_speed) in [
        (KeyCode::KeyA, true, -220.0),
        (KeyCode::KeyD, true, 0.0),
        (KeyCode::KeyA, false, 220.0),
        (KeyCode::KeyD, false, 0.0),
    ] {
        let previous_x = position(&app, player).x;
        if pressed {
            key.press(app.world_mut());
        } else {
            key.release(app.world_mut());
        }
        app.update();
        assert_eq!(velocity(&app, player), Vec2::new(expected_speed, 0.0));
        let displacement = position(&app, player).x - previous_x;
        if expected_speed == 0.0 {
            assert_eq!(displacement, 0.0);
        } else {
            assert!(displacement * expected_speed > 0.0);
        }
    }
}

#[test]
fn landing_allows_one_jump_per_grounded_space_press() {
    let (mut app, player) = game(Vec2::NEG_Y * 1000.0);
    advance(&mut app, 128);
    let ground_y = position(&app, player).y;
    // The road surface is -162.5 and the player is 50 pixels tall.
    assert!((ground_y + 137.5).abs() < 2.0);
    assert!(velocity(&app, player).y.abs() < 1.0);

    KeyCode::Space.press(app.world_mut());
    app.update();
    assert!(velocity(&app, player).y > 350.0);
    assert!(position(&app, player).y > ground_y);

    KeyCode::Space.release(app.world_mut());
    advance(&mut app, 8);
    let ascending_speed = velocity(&app, player).y;
    KeyCode::Space.press(app.world_mut());
    app.update();
    assert!(velocity(&app, player).y < ascending_speed);

    // Holding the second press through landing must not start another jump.
    advance(&mut app, 128);
    assert!((position(&app, player).y - ground_y).abs() < 2.0);
    assert!(velocity(&app, player).y.abs() < 1.0);
    KeyCode::Space.release(app.world_mut());
    app.update();
    KeyCode::Space.press(app.world_mut());
    app.update();
    assert!(velocity(&app, player).y > 350.0);
}

#[test]
fn wall_and_ceiling_probe_hits_do_not_count_as_ground() {
    for (obstacle_offset, size, probe_offset) in [
        (Vec2::new(-20.0, 0.0), Vec2::new(10.0, 200.0), Vec2::NEG_X),
        (Vec2::new(20.0, 0.0), Vec2::new(10.0, 200.0), Vec2::X),
        (Vec2::new(0.0, 30.0), Vec2::new(200.0, 10.0), Vec2::Y),
    ] {
        let (mut app, player) = game(Vec2::ZERO);
        move_away_from_level(&mut app, player);
        let obstacle = app
            .world_mut()
            .spawn((
                RigidBody::Static,
                Collider::rectangle(size.x, size.y),
                Transform::from_translation((Vec2::new(1000.0, 0.0) + obstacle_offset).extend(0.0)),
            ))
            .id();
        // The body touches the obstacle. Shift its inset probe into the surface
        // to obtain real side/ceiling normals without synthesizing ShapeHits.
        app.world_mut().get_mut::<ShapeCaster>(player).unwrap().origin = probe_offset * 2.0;
        advance(&mut app, 2);
        assert!(
            app.world()
                .get::<ShapeHits>(player)
                .unwrap()
                .iter()
                .any(|hit| hit.entity == obstacle && hit.normal2.dot(probe_offset) > 0.99)
        );

        KeyCode::Space.press(app.world_mut());
        app.update();
        assert_eq!(velocity(&app, player).y, 0.0);
    }
}

#[test]
fn a_sensor_neither_supports_jumping_nor_hides_solid_ground() {
    let (mut app, player) = game(Vec2::ZERO);
    move_away_from_level(&mut app, player);
    let sensor = app
        .world_mut()
        .spawn((
            Collider::rectangle(100.0, 20.0),
            Sensor,
            Transform::from_xyz(1000.0, -36.0, 0.0),
        ))
        .id();
    advance(&mut app, 2);
    assert!(
        app.world()
            .get::<ShapeHits>(player)
            .unwrap()
            .iter()
            .any(|hit| hit.entity == sensor)
    );
    KeyCode::Space.press(app.world_mut());
    app.update();
    assert_eq!(velocity(&app, player).y, 0.0);

    KeyCode::Space.release(app.world_mut());
    // This platform is one pixel behind the sensor along the downward cast.
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::rectangle(100.0, 20.0),
        Transform::from_xyz(1000.0, -37.0, 0.0),
    ));
    advance(&mut app, 2);
    KeyCode::Space.press(app.world_mut());
    app.update();
    assert!(velocity(&app, player).y > 350.0);
}
