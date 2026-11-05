use super::*;
use crate::{
    game::HasHelmet,
    player::{Player, Stunned, player_scene},
    test_support::{assert_removed, finish_app, headless_app, scene_app},
};
use avian2d::prelude::*;

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

    assert_removed(&app, &[object]);
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
        assert_removed(&app, &[object]);
        assert!(!app.world().resource::<HasHelmet>().0);
        assert_eq!(app.world().get::<Stunned>(player).is_none(), protected);
        if protected {
            assert_eq!(
                app.world().get::<LinearVelocity>(player).unwrap().0,
                velocity
            );
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
            let mut target = app
                .world_mut()
                .spawn((Collider::rectangle(200.0, 20.0), Transform::default()));
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
