use super::*;
use crate::physics::GameLayer;

#[test]
fn stunned_player_keeps_falling_under_gravity() {
    let mut app = headless_app(Vec2::NEG_Y * 1000.0);
    app.add_plugins(PlayerPlugin);
    let player = app
        .world_mut()
        .spawn_scene(player_scene(Vec2::new(0.0, 200.0)))
        .unwrap()
        .insert(Stunned { remaining: 2.0 })
        .id();
    finish_app(&mut app);
    KeyCode::KeyD.press(app.world_mut());
    KeyCode::ShiftLeft.press(app.world_mut());
    KeyCode::Space.press(app.world_mut());
    let stamina = app.world().resource::<Stamina>().0;

    advance(&mut app, 16);

    let velocity = app.world().get::<LinearVelocity>(player).unwrap();
    assert_eq!(velocity.x, 0.0);
    assert!(velocity.y < 0.0);
    assert!(app.world().get::<Position>(player).unwrap().y < 200.0);
    assert!(app.world().get::<Stunned>(player).is_some());
    assert_eq!(app.world().resource::<Stamina>().0, stamina);
}

#[test]
fn keyboard_controls_move_sprint_and_jump_after_a_real_landing() {
    let mut app = headless_app(Vec2::NEG_Y * 1000.0);
    {
        let mut settings = app.world_mut().resource_mut::<GameSettings>();
        settings.multiplier = 2.5;
        settings.per_second = 8.0;
    }
    app.add_plugins(PlayerPlugin);
    let player = app
        .world_mut()
        .spawn_scene(player_scene(Vec2::new(0.0, 80.0)))
        .unwrap()
        .id();
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::rectangle(2000.0, 20.0),
        CollisionLayers::new(GameLayer::Ground, LayerMask::ALL),
        Friction::ZERO,
        Transform::default(),
    ));
    finish_app(&mut app);

    let landed = |app: &App| {
        app.world().get::<Position>(player).unwrap().y < 80.0
            && app.world().get::<LinearVelocity>(player).unwrap().y.abs() < 1.0
            && app
                .world()
                .get::<ShapeHits>(player)
                .unwrap()
                .iter()
                .any(|hit| (-hit.normal2).dot(Vec2::Y) > 0.7)
    };
    for _ in 0..240 {
        if landed(&app) {
            break;
        }
        app.update();
    }
    assert!(
        landed(&app),
        "player did not land within the simulation limit"
    );

    KeyCode::KeyA.press(app.world_mut());
    app.update();
    assert!(app.world().get::<LinearVelocity>(player).unwrap().x < 0.0);

    KeyCode::KeyA.release(app.world_mut());
    KeyCode::KeyD.press(app.world_mut());
    app.update();
    let walking_speed = app.world().get::<LinearVelocity>(player).unwrap().x;
    assert!(walking_speed > 0.0);

    let stamina_before_sprint = app.world().resource::<Stamina>().0;
    KeyCode::ShiftLeft.press(app.world_mut());
    app.update();
    assert!(app.world().get::<LinearVelocity>(player).unwrap().x > walking_speed);
    assert!(app.world().resource::<Stamina>().0 < stamina_before_sprint);

    KeyCode::Space.press(app.world_mut());
    app.update();
    assert!(app.world().get::<LinearVelocity>(player).unwrap().y > 0.0);
}
