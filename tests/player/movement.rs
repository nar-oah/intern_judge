use super::*;

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
