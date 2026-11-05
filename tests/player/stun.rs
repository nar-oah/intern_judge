use super::*;

#[test]
fn stun_blocks_movement_sprint_and_jump_without_spending_stamina() {
    let mut movement = Movement::new(50.0);
    movement
        .world
        .entity_mut(movement.player)
        .insert(Stunned { remaining: 2.0 });
    movement.contacts(&[Vec2::Y]);
    movement.actions(&[
        PlayerAction::Right,
        PlayerAction::Sprint,
        PlayerAction::Jump,
    ]);

    movement.step(0.5);

    assert_eq!(movement.velocity(), Vec2::new(0.0, -70.0));
    assert_eq!(movement.stamina(), 50.0);
    assert_eq!(
        movement
            .world
            .get::<Stunned>(movement.player)
            .unwrap()
            .remaining,
        1.5
    );
}

#[test]
fn expired_stun_restores_movement_on_the_same_fixed_step() {
    let mut movement = Movement::new(50.0);
    movement
        .world
        .entity_mut(movement.player)
        .insert(Stunned { remaining: 2.0 });
    movement.actions(&[PlayerAction::Right]);

    movement.step(1.5);
    assert_eq!(movement.velocity(), Vec2::new(0.0, -70.0));
    movement.step(0.5);

    assert!(movement.world.get::<Stunned>(movement.player).is_none());
    assert_eq!(movement.velocity(), Vec2::new(WALK_SPEED, -70.0));
}
