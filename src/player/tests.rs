use super::*;
use crate::physics::GameLayer;
use crate::test_support::{advance, finish_app, headless_app};
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

const PLAYER_SPAWN: Vec2 = Vec2::new(125.0, 80.0);

fn game(gravity: Vec2) -> (App, Entity) {
    let mut app = headless_app(gravity);
    app.add_plugins(PlayerPlugin);
    let player = app
        .world_mut()
        .spawn_scene(player_scene(PLAYER_SPAWN))
        .unwrap()
        .id();
    finish_app(&mut app);
    (app, player)
}

fn position(app: &App, player: Entity) -> Vec2 {
    app.world().get::<Position>(player).unwrap().0
}

fn velocity(app: &App, player: Entity) -> Vec2 {
    app.world().get::<LinearVelocity>(player).unwrap().0
}

fn ground(app: &mut App, center: Vec2, size: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            RigidBody::Static,
            Collider::rectangle(size.x, size.y),
            CollisionLayers::new(GameLayer::Ground, LayerMask::ALL),
            Transform::from_translation(center.extend(0.0)),
        ))
        .id()
}

#[test]
fn player_scene_initializes_rendering_physics_and_keyboard_input() {
    let (mut app, player) = game(Vec2::ZERO);
    let entity = app.world().entity(player);
    assert_eq!(position(&app, player), PLAYER_SPAWN);
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
    assert_eq!(
        entity.get::<ShapeCaster>().unwrap().query_filter.mask,
        LayerMask::from(GameLayer::Ground)
    );
    assert!(entity.contains::<ShapeHits>());
    assert!(entity.contains::<InputMap<PlayerAction>>());

    KeyCode::KeyE.press(app.world_mut());
    KeyCode::ShiftLeft.press(app.world_mut());
    app.update();
    assert!(
        app.world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap()
            .just_pressed(&PlayerAction::Interact)
    );
    assert!(
        app.world()
            .get::<ActionState<PlayerAction>>(player)
            .unwrap()
            .just_pressed(&PlayerAction::Sprint)
    );
}

#[test]
fn player_plugin_runs_without_spawning_a_player() {
    let mut app = headless_app(Vec2::ZERO);
    app.add_plugins(PlayerPlugin);
    finish_app(&mut app);

    KeyCode::KeyD.press(app.world_mut());
    KeyCode::Space.press(app.world_mut());
    advance(&mut app, 2);

    let world = app.world_mut();
    assert_eq!(world.query::<&Player>().iter(world).count(), 0);
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
fn sprinting_scales_speed_and_drains_five_stamina_per_second_at_any_fixed_timestep() {
    for frames_per_second in [32, 64] {
        let (mut app, player) = game(Vec2::ZERO);
        let timestep = Duration::from_secs_f64(1.0 / f64::from(frames_per_second));
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .set_timestep(timestep);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(timestep));
        app.world_mut().get_mut::<LinearVelocity>(player).unwrap().y = -70.0;
        KeyCode::KeyA.press(app.world_mut());
        KeyCode::ShiftLeft.press(app.world_mut());

        advance(&mut app, frames_per_second as usize);

        assert_eq!(velocity(&app, player), Vec2::new(-330.0, -70.0));
        assert!((app.world().resource::<Stamina>().0 - 95.0).abs() < 0.001);
    }
}

#[test]
fn stamina_is_unchanged_without_shift_or_horizontal_movement() {
    let (mut app, player) = game(Vec2::ZERO);
    app.world_mut().resource_mut::<Stamina>().0 = 50.0;

    // Shift alone does not consume stamina.
    KeyCode::ShiftLeft.press(app.world_mut());
    advance(&mut app, 64);
    assert_eq!(velocity(&app, player).x, 0.0);
    assert_eq!(app.world().resource::<Stamina>().0, 50.0);

    // Opposing movement inputs also leave the player stationary.
    KeyCode::KeyA.press(app.world_mut());
    KeyCode::KeyD.press(app.world_mut());
    advance(&mut app, 64);
    assert_eq!(velocity(&app, player).x, 0.0);
    assert_eq!(app.world().resource::<Stamina>().0, 50.0);

    // Releasing Shift restores walking without spending or recovering stamina.
    KeyCode::ShiftLeft.release(app.world_mut());
    KeyCode::KeyA.release(app.world_mut());
    advance(&mut app, 64);
    assert_eq!(velocity(&app, player).x, 220.0);
    assert_eq!(app.world().resource::<Stamina>().0, 50.0);

    KeyCode::KeyD.release(app.world_mut());
    advance(&mut app, 64);
    assert_eq!(velocity(&app, player).x, 0.0);
    assert_eq!(app.world().resource::<Stamina>().0, 50.0);
}

#[test]
fn depleted_stamina_preserves_walking_and_jumping() {
    let (mut app, player) = game(Vec2::NEG_Y * 1000.0);
    let floor_center = position(&app, player) + Vec2::new(0.0, -100.0);
    let floor = ground(&mut app, floor_center, Vec2::new(400.0, 20.0));
    app.world_mut().entity_mut(floor).insert(Friction::ZERO);
    advance(&mut app, 128);
    app.world_mut().resource_mut::<Stamina>().0 = 0.01;

    KeyCode::KeyD.press(app.world_mut());
    KeyCode::ShiftLeft.press(app.world_mut());
    app.update();
    assert_eq!(velocity(&app, player).x, 330.0);
    assert_eq!(app.world().resource::<Stamina>().0, 0.0);

    KeyCode::Space.press(app.world_mut());
    app.update();
    assert_eq!(velocity(&app, player).x, 220.0);
    assert!(velocity(&app, player).y > 350.0);
    assert_eq!(app.world().resource::<Stamina>().0, 0.0);

    advance(&mut app, 64);
    assert_eq!(velocity(&app, player).x, 220.0);
    assert_eq!(app.world().resource::<Stamina>().0, 0.0);
}

#[test]
fn landing_allows_one_jump_per_grounded_space_press() {
    let (mut app, player) = game(Vec2::NEG_Y * 1000.0);
    let floor_size = Vec2::new(400.0, 20.0);
    let floor_center = position(&app, player) + Vec2::new(0.0, -100.0);
    let player_height = app
        .world()
        .get::<Sprite>(player)
        .unwrap()
        .custom_size
        .unwrap()
        .y;
    let expected_ground_y = floor_center.y + (floor_size.y + player_height) / 2.0;
    ground(&mut app, floor_center, floor_size);
    advance(&mut app, 128);
    let ground_y = position(&app, player).y;
    assert!((ground_y - expected_ground_y).abs() < 2.0);
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
        let obstacle_center = position(&app, player) + obstacle_offset;
        let obstacle = ground(&mut app, obstacle_center, size);
        // The body touches the obstacle. Shift its inset probe into the surface
        // to obtain real side/ceiling normals without synthesizing ShapeHits.
        app.world_mut()
            .get_mut::<ShapeCaster>(player)
            .unwrap()
            .origin = probe_offset * 2.0;
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
fn non_ground_colliders_neither_support_jumping_nor_hide_ground() {
    for is_sensor in [false, true] {
        let (mut app, player) = game(Vec2::ZERO);
        let player_position = position(&app, player);
        let mut collider = app.world_mut().spawn((
            RigidBody::Static,
            Collider::rectangle(100.0, 20.0),
            CollisionLayers::new(GameLayer::Default, LayerMask::ALL),
            Transform::from_translation((player_position + Vec2::new(0.0, -36.0)).extend(0.0)),
        ));
        if is_sensor {
            collider.insert(Sensor);
        }
        let ignored = collider.id();
        advance(&mut app, 2);
        assert!(
            app.world()
                .get::<ShapeHits>(player)
                .unwrap()
                .iter()
                .all(|hit| hit.entity != ignored)
        );
        KeyCode::Space.press(app.world_mut());
        app.update();
        assert_eq!(velocity(&app, player).y, 0.0);

        KeyCode::Space.release(app.world_mut());
        // The ground is behind the ignored collider along the downward cast.
        let platform = ground(
            &mut app,
            player_position + Vec2::new(0.0, -37.0),
            Vec2::new(100.0, 20.0),
        );
        advance(&mut app, 2);
        assert!(
            app.world()
                .get::<ShapeHits>(player)
                .unwrap()
                .iter()
                .any(|hit| hit.entity == platform)
        );
        KeyCode::Space.press(app.world_mut());
        app.update();
        assert!(velocity(&app, player).y > 350.0);
    }
}

#[test]
fn airborne_movement_and_jump_input_preserve_vertical_velocity() {
    let (mut app, player) = game(Vec2::ZERO);
    app.world_mut().get_mut::<LinearVelocity>(player).unwrap().y = -70.0;

    KeyCode::KeyD.press(app.world_mut());
    KeyCode::Space.press(app.world_mut());
    app.update();
    assert_eq!(velocity(&app, player), Vec2::new(220.0, -70.0));

    KeyCode::KeyD.release(app.world_mut());
    app.update();
    assert_eq!(velocity(&app, player), Vec2::new(0.0, -70.0));
}
