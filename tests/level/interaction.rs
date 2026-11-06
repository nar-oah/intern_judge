use crate::level::{Interactable, PausedInteractRequested, RunningInteractRequested, interaction};
use crate::{
    game::CommuteState,
    player::{Player, PlayerAction},
};
use avian2d::prelude::CollidingEntities;
use bevy::{platform::time::Instant, prelude::*};
use leafwing_input_manager::prelude::*;

#[derive(Resource, Default)]
struct Requests {
    running: Vec<Entity>,
    paused: Vec<(Entity, Entity)>,
}

fn interaction_app() -> App {
    let mut app = App::new();
    app.init_resource::<Requests>()
        .insert_resource(State::new(CommuteState::Running))
        .add_systems(PostUpdate, interaction::interact)
        .add_observer(
            |event: On<RunningInteractRequested>, mut requests: ResMut<Requests>| {
                requests.running.push(event.entity);
            },
        )
        .add_observer(
            |event: On<PausedInteractRequested>, mut requests: ResMut<Requests>| {
                requests.paused.push((event.entity, event.player));
            },
        );
    app
}

fn player(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((Player::default(), ActionState::<PlayerAction>::default()))
        .id()
}

fn target(app: &mut App, collisions: &[Entity]) -> Entity {
    app.world_mut()
        .spawn((
            Interactable,
            CollidingEntities(collisions.iter().copied().collect()),
        ))
        .id()
}

fn interact(app: &mut App, player: Entity, pressed: bool) {
    let mut actions = app
        .world_mut()
        .get_mut::<ActionState<PlayerAction>>(player)
        .unwrap();
    if pressed {
        actions.press(&PlayerAction::Interact);
    } else {
        actions.release(&PlayerAction::Interact);
    }
}

fn frame(app: &mut App) {
    app.update();
    // The input manager is absent here; consume the injected button edges ourselves.
    let now = Instant::now();
    let world = app.world_mut();
    for mut actions in world
        .query::<&mut ActionState<PlayerAction>>()
        .iter_mut(world)
    {
        actions.tick(now, now);
    }
}

#[test]
fn interaction_requires_overlap_at_the_time_of_a_fresh_press() {
    let mut app = interaction_app();
    let player = player(&mut app);
    let target = target(&mut app, &[]);

    interact(&mut app, player, true);
    frame(&mut app);
    app.world_mut()
        .get_mut::<CollidingEntities>(target)
        .unwrap()
        .insert(player);
    frame(&mut app);
    assert!(app.world().resource::<Requests>().running.is_empty());

    interact(&mut app, player, false);
    frame(&mut app);
    interact(&mut app, player, true);
    frame(&mut app);
    frame(&mut app);
    assert_eq!(app.world().resource::<Requests>().running, [target]);

    interact(&mut app, player, false);
    frame(&mut app);
    interact(&mut app, player, true);
    frame(&mut app);
    assert_eq!(app.world().resource::<Requests>().running, [target, target]);
}

#[test]
fn interaction_routes_fresh_presses_to_the_current_commute_state() {
    let mut app = interaction_app();
    let player = player(&mut app);
    let target = target(&mut app, &[player]);

    for commute in [
        CommuteState::Running,
        CommuteState::Paused,
        CommuteState::Running,
    ] {
        app.insert_resource(State::new(commute));
        interact(&mut app, player, true);
        frame(&mut app);
        interact(&mut app, player, false);
        frame(&mut app);
    }

    let requests = app.world().resource::<Requests>();
    assert_eq!(requests.running, [target, target]);
    assert_eq!(requests.paused, [(target, player)]);
}

#[test]
fn non_interactive_collisions_and_non_player_input_do_not_request_interactions() {
    let mut app = interaction_app();
    let player = player(&mut app);
    let non_player = app
        .world_mut()
        .spawn(ActionState::<PlayerAction>::default())
        .id();
    target(&mut app, &[non_player]);
    app.world_mut()
        .spawn(CollidingEntities([player].into_iter().collect()));
    interact(&mut app, player, true);
    interact(&mut app, non_player, true);

    frame(&mut app);
    assert!(app.world().resource::<Requests>().running.is_empty());
    assert!(app.world().resource::<Requests>().paused.is_empty());

    app.world_mut().despawn(player);
    frame(&mut app);
    assert!(app.world().resource::<Requests>().running.is_empty());
    assert!(app.world().resource::<Requests>().paused.is_empty());
}
