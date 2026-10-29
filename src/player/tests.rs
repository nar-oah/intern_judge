use super::*;
use crate::physics::GameLayer;
use crate::test_support::{finish_app, headless_app};
use bevy::{ecs::system::RunSystemOnce, platform::time::Instant};
use std::time::Duration;

const WALK_SPEED: f32 = 200.0;
const JUMP_SPEED: f32 = 400.0;

struct Movement {
    world: World,
    player: Entity,
}

impl Movement {
    fn new(stamina: f32) -> Self {
        let mut world = World::new();
        world.insert_resource(Time::<Fixed>::default());
        world.insert_resource(GameSettings {
            commute_limit: 60.0,
            initial_money: 0,
            multiplier: 2.5,
            per_second: 8.0,
        });
        world.insert_resource(Stamina(stamina));
        let player = world
            .spawn((
                Player {
                    move_speed: WALK_SPEED,
                    jump_speed: JUMP_SPEED,
                },
                ActionState::<PlayerAction>::default(),
                ShapeHits::default(),
                LinearVelocity(Vec2::new(0.0, -70.0)),
            ))
            .id();
        Self { world, player }
    }

    fn actions(&mut self, pressed: &[PlayerAction]) {
        let mut actions = ActionState::default();
        for action in pressed {
            actions.press(action);
        }
        self.world.entity_mut(self.player).insert(actions);
    }

    fn contacts(&mut self, surface_normals: &[Vec2]) {
        let hits = surface_normals
            .iter()
            .map(|&normal| ShapeHitData {
                entity: Entity::PLACEHOLDER,
                distance: 0.0,
                point1: Vec2::ZERO,
                point2: Vec2::ZERO,
                normal1: normal,
                normal2: -normal,
            })
            .collect();
        self.world.entity_mut(self.player).insert(ShapeHits(hits));
    }

    fn step(&mut self, seconds: f32) {
        self.world
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs_f32(seconds));
        self.world.run_system_once(player_movement).unwrap();
        let now = Instant::now();
        self.world
            .get_mut::<ActionState<PlayerAction>>(self.player)
            .unwrap()
            .tick(now, now);
    }

    fn velocity(&self) -> Vec2 {
        self.world.get::<LinearVelocity>(self.player).unwrap().0
    }

    fn stamina(&self) -> f32 {
        self.world.resource::<Stamina>().0
    }
}

#[test]
fn movement_follows_direction_and_stops_when_inputs_cancel_or_release() {
    let mut movement = Movement::new(50.0);
    for (actions, expected_x) in [
        (vec![PlayerAction::Left], -WALK_SPEED),
        (vec![PlayerAction::Right], WALK_SPEED),
        (vec![PlayerAction::Left, PlayerAction::Right], 0.0),
        (vec![], 0.0),
    ] {
        movement.actions(&actions);
        movement.step(0.25);
        assert_eq!(movement.velocity(), Vec2::new(expected_x, -70.0));
    }
}

#[test]
fn sprint_uses_configured_multiplier_and_drains_by_elapsed_time() {
    for steps in [vec![1.0], vec![0.25, 0.75]] {
        let mut movement = Movement::new(20.0);
        movement.actions(&[PlayerAction::Left, PlayerAction::Sprint]);
        for seconds in steps {
            movement.step(seconds);
        }

        assert_eq!(movement.velocity(), Vec2::new(-500.0, -70.0));
        assert!((movement.stamina() - 12.0).abs() < 0.001);
    }
}

#[test]
fn walking_or_sprint_without_movement_does_not_spend_stamina() {
    let mut movement = Movement::new(50.0);
    for actions in [
        vec![],
        vec![PlayerAction::Right],
        vec![PlayerAction::Sprint],
        vec![
            PlayerAction::Left,
            PlayerAction::Right,
            PlayerAction::Sprint,
        ],
    ] {
        movement.actions(&actions);
        movement.step(1.0);
        assert_eq!(movement.stamina(), 50.0);
    }
}

#[test]
fn exhaustion_clamps_stamina_to_zero_and_still_allows_walking_and_jumping() {
    let mut movement = Movement::new(0.1);
    movement.actions(&[PlayerAction::Right, PlayerAction::Sprint]);
    movement.step(0.25);
    assert_eq!(movement.stamina(), 0.0);

    movement.contacts(&[Vec2::Y]);
    movement.actions(&[
        PlayerAction::Right,
        PlayerAction::Sprint,
        PlayerAction::Jump,
    ]);
    movement.step(0.25);
    assert_eq!(movement.velocity(), Vec2::new(WALK_SPEED, JUMP_SPEED));
    assert_eq!(movement.stamina(), 0.0);
}

#[test]
fn jumping_requires_support_below_the_player() {
    for (normals, expected_y) in [
        (vec![], -70.0),
        (vec![Vec2::X], -70.0),
        (vec![Vec2::NEG_Y], -70.0),
        (vec![Vec2::Y], JUMP_SPEED),
        (vec![Vec2::X, Vec2::Y], JUMP_SPEED),
    ] {
        let mut movement = Movement::new(50.0);
        movement.contacts(&normals);
        movement.actions(&[PlayerAction::Jump]);
        movement.step(0.25);
        assert_eq!(movement.velocity().y, expected_y, "contacts: {normals:?}");
    }
}

#[test]
fn holding_an_airborne_jump_does_not_jump_on_landing() {
    let mut movement = Movement::new(50.0);
    movement.actions(&[PlayerAction::Jump]);
    movement.step(0.25);
    assert_eq!(movement.velocity().y, -70.0);

    movement.contacts(&[Vec2::Y]);
    movement
        .world
        .get_mut::<LinearVelocity>(movement.player)
        .unwrap()
        .y = 0.0;
    movement.step(0.25);
    assert_eq!(movement.velocity().y, 0.0);

    movement.actions(&[]);
    movement.step(0.25);
    movement.actions(&[PlayerAction::Jump]);
    movement.step(0.25);
    assert_eq!(movement.velocity().y, JUMP_SPEED);
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
