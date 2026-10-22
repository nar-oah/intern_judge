use super::*;
use crate::physics::GameLayer;
use crate::player::{Player, PlayerAction, PlayerPlugin};
use crate::test_support::{advance, finish_app, headless_app};
use avian2d::prelude::*;
use bevy::{
    log::{tracing, tracing_subscriber},
    platform::time::Instant,
    state::app::StatesPlugin,
};
use leafwing_input_manager::prelude::*;
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};

#[derive(Resource, Default)]
struct Requests(Vec<(Entity, Entity)>);

fn record_requests(app: &mut App) {
    app.init_resource::<Requests>().add_observer(
        |event: On<InteractRequested>, mut requests: ResMut<Requests>| {
            requests.0.push((event.entity, event.player));
        },
    );
}

fn scene_app() -> App {
    let mut app = headless_app(Vec2::ZERO);
    app.add_plugins(LevelPlugin);
    record_requests(&mut app);
    finish_app(&mut app);
    advance(&mut app, 2);
    app
}

fn interaction_app() -> App {
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .init_state::<GamePhase>()
        .add_systems(
            PostUpdate,
            interaction::interact.run_if(in_state(GamePhase::Exploring)),
        );
    record_requests(&mut app);
    app
}

fn single<C: Component>(app: &mut App) -> Entity {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<C>>()
        .single(world)
        .unwrap()
}

fn teleport(app: &mut App, entity: Entity, position: Vec3) {
    app.world_mut()
        .get_mut::<Transform>(entity)
        .unwrap()
        .translation = position;
    advance(app, 3);
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

fn set_interact(app: &mut App, player: Entity, pressed: bool) {
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

fn interaction_frame(app: &mut App) {
    app.update();
    // These tests supply action states directly, without the input manager plugin.
    let now = Instant::now();
    let world = app.world_mut();
    for mut actions in world
        .query::<&mut ActionState<PlayerAction>>()
        .iter_mut(world)
    {
        actions.tick(now, now);
    }
}

fn interact_key(app: &mut App, pressed: bool) {
    if pressed {
        KeyCode::KeyE.press(app.world_mut());
    } else {
        KeyCode::KeyE.release(app.world_mut());
    }
    app.update();
}

fn requests(app: &App) -> &[(Entity, Entity)] {
    &app.world().resource::<Requests>().0
}

#[test]
fn road_scene_keeps_physics_aligned_and_despawns_its_children() {
    let mut app = scene_app();
    let road = single::<RoadId>(&mut app);
    let player = player(&mut app);
    assert_eq!(
        app.world().get::<RoadId>(road),
        Some(&RoadId("example_road"))
    );
    let children = app.world().get::<Children>(road).unwrap().to_vec();
    assert!(!children.is_empty());
    let offset = Vec3::new(1000.0, 200.0, 0.0);
    teleport(&mut app, road, offset);

    let mut platforms = 0;
    let mut buildings = Vec::new();
    let mut switches = Vec::new();
    for &child in &children {
        let world = app.world();
        assert_eq!(world.get::<ChildOf>(child).unwrap().parent(), road);
        let size = world.get::<Sprite>(child).unwrap().custom_size.unwrap();
        let collider = world.get::<Collider>(child).unwrap();
        let half_extents = collider.shape().as_cuboid().unwrap().half_extents;
        assert_eq!(Vec2::new(half_extents.x, half_extents.y) * 2.0, size);
        let expected = world.get::<Transform>(child).unwrap().translation + offset;
        assert!(
            world
                .get::<GlobalTransform>(child)
                .unwrap()
                .translation()
                .abs_diff_eq(expected, 0.001)
        );
        assert!(
            world
                .get::<Position>(child)
                .unwrap()
                .0
                .abs_diff_eq(expected.truncate(), 0.001)
        );
        if let Some(platform) = world.get::<Platform>(child) {
            platforms += 1;
            assert_eq!(platform.size, size);
            assert_eq!(world.get::<RigidBody>(child), Some(&RigidBody::Static));
            assert_eq!(
                world.get::<CollisionLayers>(child).unwrap().memberships,
                LayerMask::from(GameLayer::Ground)
            );
        } else {
            assert!(world.get::<Interactable>(child).is_some());
            assert!(world.get::<Sensor>(child).is_some());
            assert!(world.get::<CollidingEntities>(child).is_some());
            if let Some(kind) = world.get::<BuildingType>(child) {
                buildings.push(*kind);
            }
            if let Some(point) = world.get::<SwitchPoint>(child) {
                switches.push(point.destinations);
            }
        }
    }
    assert!(platforms > 0);
    for kind in [
        BuildingType::Residential,
        BuildingType::Shop,
        BuildingType::Factory,
    ] {
        assert!(buildings.contains(&kind));
    }
    assert_eq!(switches, [&[NEXT_ROAD, COURT][..]]);

    let world = app.world_mut();
    assert!(world.despawn(road));
    assert!(world.get_entity(road).is_err());
    assert!(
        children
            .into_iter()
            .all(|child| world.get_entity(child).is_err())
    );
    assert!(world.get::<Player>(player).is_some());
    assert_eq!(world.query::<&Interactable>().iter(world).count(), 0);
}

#[test]
fn interactions_request_all_overlaps_only_on_a_fresh_press() {
    let mut app = interaction_app();
    let player = player(&mut app);
    let targets = [target(&mut app, &[player]), target(&mut app, &[player])];

    interaction_frame(&mut app);
    assert!(requests(&app).is_empty(), "overlap alone does not interact");
    set_interact(&mut app, player, true);
    interaction_frame(&mut app);
    assert_eq!(requests(&app).len(), targets.len());
    for entity in targets {
        assert!(requests(&app).contains(&(entity, player)));
    }
    interaction_frame(&mut app);
    assert_eq!(requests(&app).len(), 2, "holding interact does not repeat");

    for entity in targets {
        app.world_mut()
            .get_mut::<CollidingEntities>(entity)
            .unwrap()
            .clear();
    }
    set_interact(&mut app, player, false);
    interaction_frame(&mut app);
    set_interact(&mut app, player, true);
    interaction_frame(&mut app);
    assert_eq!(
        requests(&app).len(),
        2,
        "a press without overlap does nothing"
    );
    for entity in targets {
        app.world_mut()
            .get_mut::<CollidingEntities>(entity)
            .unwrap()
            .insert(player);
    }
    interaction_frame(&mut app);
    assert_eq!(requests(&app).len(), 2, "entering while held does nothing");
    set_interact(&mut app, player, false);
    interaction_frame(&mut app);
    set_interact(&mut app, player, true);
    interaction_frame(&mut app);
    assert_eq!(requests(&app).len(), 4);
}

#[test]
fn interactions_filter_each_players_action_and_ignore_unmarked_targets_and_non_players() {
    let mut app = interaction_app();
    let first = player(&mut app);
    let idle = player(&mut app);
    let second = player(&mut app);
    let mut actions = ActionState::<PlayerAction>::default();
    actions.press(&PlayerAction::Interact);
    let non_player = app.world_mut().spawn(actions).id();
    let shared = target(&mut app, &[first, idle, second, non_player]);
    let private = target(&mut app, &[first]);
    target(&mut app, &[non_player]);
    app.world_mut()
        .spawn(CollidingEntities([first].into_iter().collect()));
    app.world_mut().spawn(Interactable);
    app.world_mut()
        .get_mut::<ActionState<PlayerAction>>(idle)
        .unwrap()
        .press(&PlayerAction::Right);
    set_interact(&mut app, first, true);
    set_interact(&mut app, second, true);

    interaction_frame(&mut app);

    assert_eq!(requests(&app).len(), 3);
    for request in [(shared, first), (shared, second), (private, first)] {
        assert!(requests(&app).contains(&request));
    }
}

#[test]
fn e_interacts_with_scene_sensors_only_during_physical_overlap() {
    let mut app = headless_app(Vec2::ZERO);
    app.add_plugins((PlayerPlugin, LevelPlugin));
    record_requests(&mut app);
    finish_app(&mut app);
    advance(&mut app, 2);
    let road = single::<RoadId>(&mut app);
    let player = single::<Player>(&mut app);
    teleport(&mut app, road, Vec3::new(1000.0, 200.0, 0.0));
    let world = app.world_mut();
    let shop = world
        .query::<(Entity, &BuildingType)>()
        .iter(world)
        .find(|(_, kind)| **kind == BuildingType::Shop)
        .unwrap()
        .0;
    let center = world.get::<GlobalTransform>(shop).unwrap().translation();
    let shop_size = world.get::<Sprite>(shop).unwrap().custom_size.unwrap();
    let player_half_width = world
        .get::<Collider>(player)
        .unwrap()
        .shape()
        .as_cuboid()
        .unwrap()
        .half_extents
        .x;
    let outside = center + Vec3::X * (shop_size.x / 2.0 + player_half_width + 5.0);
    teleport(&mut app, player, outside);
    assert!(
        !app.world()
            .get::<CollidingEntities>(shop)
            .unwrap()
            .contains(&player)
    );
    interact_key(&mut app, true);
    assert!(requests(&app).is_empty(), "nearby centers are insufficient");

    interact_key(&mut app, false);
    teleport(&mut app, player, center);
    assert!(
        app.world()
            .get::<CollidingEntities>(shop)
            .unwrap()
            .contains(&player)
    );
    assert!(requests(&app).is_empty(), "overlap alone does not interact");
    interact_key(&mut app, true);
    assert_eq!(requests(&app), [(shop, player)]);
    advance(&mut app, 3);
    assert_eq!(requests(&app).len(), 1, "holding E does not repeat");

    teleport(&mut app, player, outside);
    assert!(
        !app.world()
            .get::<CollidingEntities>(shop)
            .unwrap()
            .contains(&player)
    );
}

#[derive(Clone, Default)]
struct LogOutput(Arc<Mutex<Vec<u8>>>);

impl Write for LogOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn building_scenes_bind_their_own_interaction_observers() {
    // Keep scoped logging interest visible when other test threads register callsites.
    let _other_dispatch = tracing::Dispatch::new(tracing_subscriber::registry());
    let output = LogOutput::default();
    let writer = output.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_ansi(false)
        .without_time()
        .with_target(false)
        .with_writer(move || writer.clone())
        .finish();
    tracing::subscriber::with_default(subscriber, || {
        let mut app = scene_app();
        let player = player(&mut app);
        let world = app.world_mut();
        let targets: Vec<_> = world
            .query::<(Entity, &BuildingType)>()
            .iter(world)
            .map(|(entity, kind)| (entity, format!("Building interaction: {kind:?}")))
            .collect();
        assert!(!targets.is_empty());

        for (entity, expected) in targets {
            output.0.lock().unwrap().clear();
            app.world_mut()
                .trigger(InteractRequested { entity, player });
            let logs = String::from_utf8(output.0.lock().unwrap().clone()).unwrap();
            let interactions: Vec<_> = logs
                .lines()
                .filter(|line| line.contains("Building interaction:"))
                .collect();
            assert_eq!(interactions.len(), 1, "captured logs:\n{logs}");
            assert!(
                interactions[0].contains(&expected),
                "captured logs:\n{logs}"
            );
        }
    });
}
