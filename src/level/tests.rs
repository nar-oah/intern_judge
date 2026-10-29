use super::{InteractRequested, Interactable, interaction};
use crate::player::{Player, PlayerAction};
use avian2d::prelude::CollidingEntities;
use bevy::{platform::time::Instant, prelude::*};
use leafwing_input_manager::prelude::*;

#[derive(Resource, Default)]
struct Requests(Vec<(Entity, Entity)>);

fn interaction_app() -> App {
    let mut app = App::new();
    app.init_resource::<Requests>()
        .add_systems(PostUpdate, interaction::interact)
        .add_observer(
            |event: On<InteractRequested>, mut requests: ResMut<Requests>| {
                requests.0.push((event.entity, event.player));
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
    assert!(app.world().resource::<Requests>().0.is_empty());

    interact(&mut app, player, false);
    frame(&mut app);
    interact(&mut app, player, true);
    frame(&mut app);
    frame(&mut app);
    assert_eq!(app.world().resource::<Requests>().0, [(target, player)]);

    interact(&mut app, player, false);
    frame(&mut app);
    interact(&mut app, player, true);
    frame(&mut app);
    assert_eq!(
        app.world().resource::<Requests>().0,
        [(target, player), (target, player)]
    );
}

#[test]
fn one_press_requests_only_one_of_the_overlapping_targets() {
    let mut app = interaction_app();
    let player = player(&mut app);
    let targets = [target(&mut app, &[player]), target(&mut app, &[player])];

    interact(&mut app, player, true);
    frame(&mut app);

    let requests = &app.world().resource::<Requests>().0;
    assert_eq!(requests.len(), 1);
    assert!(targets.contains(&requests[0].0));
    assert_eq!(requests[0].1, player);
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
    assert!(app.world().resource::<Requests>().0.is_empty());

    app.world_mut().despawn(player);
    frame(&mut app);
    assert!(app.world().resource::<Requests>().0.is_empty());
}
