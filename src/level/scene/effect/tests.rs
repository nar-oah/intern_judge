use super::*;
use crate::{
    game::HasHelmet,
    level::{RoadEffect, RoadId, scene::road_scene},
    player::{Player, PlayerPlugin, Stunned, player_scene},
    test_support::{advance, finish_app, headless_app, scene_app},
};
use avian2d::prelude::*;
use leafwing_input_manager::prelude::*;

fn impact(app: &mut App, object: Entity, target: Entity) {
    app.world_mut().trigger(CollisionStart {
        collider1: object,
        collider2: target,
        body1: Some(object),
        body2: Some(target),
    });
    app.world_mut().flush();
}

#[test]
fn impact_stuns_for_two_seconds_and_only_clears_horizontal_velocity() {
    let mut app = scene_app();
    app.init_resource::<HasHelmet>();
    let player = app
        .world_mut()
        .spawn((Player::default(), LinearVelocity(Vec2::new(220.0, -70.0))))
        .id();
    let object = app
        .world_mut()
        .spawn_scene(falling_object_scene(Vec2::Y))
        .unwrap()
        .id();

    impact(&mut app, object, player);

    assert!(app.world().get_entity(object).is_err());
    assert_eq!(app.world().get::<Stunned>(player).unwrap().remaining, 2.0);
    assert_eq!(
        app.world().get::<LinearVelocity>(player).unwrap().0,
        Vec2::new(0.0, -70.0)
    );
}

#[test]
fn helmet_is_consumed_by_one_player_hit_and_the_next_hit_stuns() {
    let mut app = scene_app();
    app.insert_resource(HasHelmet(true));
    let velocity = Vec2::new(220.0, 100.0);
    let player = app
        .world_mut()
        .spawn((Player::default(), LinearVelocity(velocity)))
        .id();
    for protected in [true, false] {
        let object = app
            .world_mut()
            .spawn_scene(falling_object_scene(Vec2::Y))
            .unwrap()
            .id();
        impact(&mut app, object, player);
        assert!(app.world().get_entity(object).is_err());
        assert!(!app.world().resource::<HasHelmet>().0);
        assert_eq!(app.world().get::<Stunned>(player).is_none(), protected);
        if protected {
            assert_eq!(app.world().get::<LinearVelocity>(player).unwrap().0, velocity);
        }
    }
}

#[test]
fn falling_object_disappears_on_real_player_platform_and_sensor_contacts() {
    for (target_kind, helmet) in [
        ("player", false),
        ("player", true),
        ("platform", true),
        ("building", true),
    ] {
        let mut app = headless_app(Vec2::NEG_Y * 1000.0);
        app.insert_resource(HasHelmet(helmet));
        let target = if target_kind == "player" {
            app.world_mut()
                .spawn_scene(player_scene(Vec2::ZERO))
                .unwrap()
                .id()
        } else {
            let mut target = app.world_mut().spawn((
                Collider::rectangle(200.0, 20.0),
                Transform::default(),
            ));
            if target_kind == "building" {
                target.insert(Sensor);
            } else {
                target.insert(RigidBody::Static);
            }
            target.id()
        };
        let object = app
            .world_mut()
            .spawn_scene(falling_object_scene(Vec2::new(0.0, 80.0)))
            .unwrap()
            .id();
        if target_kind == "player" {
            // Keep the target below the object until contact, without movement input.
            app.world_mut().entity_mut(target).insert(GravityScale(0.0));
        }
        finish_app(&mut app);
        for _ in 0..120 {
            if app.world().get_entity(object).is_err() {
                break;
            }
            app.update();
        }
        assert!(
            app.world().get_entity(object).is_err(),
            "falling object did not disappear on {target_kind} contact"
        );
        assert!(app.world().get_entity(target).is_ok());
        assert_eq!(
            app.world().get::<Stunned>(target).is_some(),
            target_kind == "player" && !helmet
        );
        assert_eq!(
            app.world().resource::<HasHelmet>().0,
            target_kind != "player"
        );
    }
}

#[test]
fn market_detour_blocks_walking_and_can_be_cleared_by_the_initial_jump() {
    let mut app = headless_app(Vec2::NEG_Y * 1000.0);
    app.add_plugins(PlayerPlugin);
    app.world_mut()
        .spawn_scene(road_scene(
            RoadId::MarketRoad,
            &[RoadEffect::ConstructionDetour],
        ))
        .unwrap();
    let world = app.world_mut();
    let player = world.query_filtered::<Entity, With<Player>>().single(world).unwrap();
    let detour = world
        .query_filtered::<Entity, With<ConstructionDetour>>()
        .single(world)
        .unwrap();
    let detour_x = world.get::<Transform>(detour).unwrap().translation.x;
    let mut position = world.get_mut::<Transform>(player).unwrap();
    position.translation.x = detour_x - 100.0;
    finish_app(&mut app);
    advance(&mut app, 30);

    KeyCode::KeyD.press(app.world_mut());
    advance(&mut app, 60);
    let blocked_x = app.world().get::<Position>(player).unwrap().x;
    assert!(blocked_x < detour_x);
    assert!(blocked_x > detour_x - 100.0);

    // Give the player a short run-up so their feet clear the obstacle before contact.
    KeyCode::KeyD.release(app.world_mut());
    KeyCode::KeyA.press(app.world_mut());
    advance(&mut app, 16);
    KeyCode::KeyA.release(app.world_mut());
    KeyCode::KeyD.press(app.world_mut());
    KeyCode::Space.press(app.world_mut());
    app.update();
    assert!(app.world().get::<LinearVelocity>(player).unwrap().y > 0.0);
    advance(&mut app, 70);
    assert!(app.world().get::<Position>(player).unwrap().x > detour_x + 60.0);
    assert!(app.world().get_entity(detour).is_ok());
}
