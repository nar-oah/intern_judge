use super::*;
use crate::player::{Player, PlayerPlugin};
use avian2d::prelude::*;
use bevy::{
    asset::AssetPlugin,
    ecs::schedule::SingleThreadedExecutor,
    input::InputPlugin,
    log::{tracing, tracing_subscriber},
    scene::ScenePlugin,
    time::TimeUpdateStrategy,
};
use leafwing_input_manager::prelude::*;
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    time::Duration,
};

fn headless_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        AssetPlugin::default(),
        ScenePlugin,
        InputPlugin,
        PhysicsPlugins::default().with_length_unit(20.0),
        PlayerPlugin,
        LevelPlugin,
    ))
    .insert_resource(Gravity(Vec2::ZERO))
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_micros(
        15_625,
    )))
    .edit_schedule(PostUpdate, |schedule| {
        // Keep interaction logs on this test's scoped tracing subscriber.
        schedule.set_executor(SingleThreadedExecutor::new());
    });
    app.finish();
    app.cleanup();
    app.update();
    app
}

fn road_entity(app: &mut App) -> Entity {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<RoadId>>()
        .single(world)
        .unwrap()
}

fn player_entity(app: &mut App) -> Entity {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<Player>>()
        .single(world)
        .unwrap()
}

fn advance_physics(app: &mut App) {
    // Allow transform propagation, a fixed physics step, and deferred observers.
    for _ in 0..3 {
        app.update();
    }
}

#[test]
fn road_scene_groups_all_parts_and_preserves_platform_dimensions() {
    let mut app = headless_app();
    let root = road_entity(&mut app);
    assert_eq!(
        app.world().get::<RoadId>(root),
        Some(&RoadId("example_road"))
    );
    let children = app.world().get::<Children>(root).unwrap().to_vec();
    assert_eq!(children.len(), 10);
    for child in &children {
        assert_eq!(app.world().get::<ChildOf>(*child).unwrap().parent(), root);
    }

    let world = app.world_mut();
    let mut platforms = world.query::<(&Platform, &Sprite, &Collider, &RigidBody)>();
    assert_eq!(platforms.iter(world).count(), 5);
    for (platform, sprite, collider, rigid_body) in platforms.iter(world) {
        assert_eq!(sprite.custom_size, Some(platform.size));
        let half_extents = collider.shape().as_cuboid().unwrap().half_extents;
        assert_eq!(
            Vec2::new(half_extents.x, half_extents.y) * 2.0,
            platform.size
        );
        assert_eq!(*rigid_body, RigidBody::Static);
    }
    let buildings: Vec<_> = world
        .query::<&BuildingType>()
        .iter(world)
        .copied()
        .collect();
    assert_eq!(buildings.len(), 3);
    for kind in [
        BuildingType::Residential,
        BuildingType::Shop,
        BuildingType::Factory,
    ] {
        assert!(buildings.contains(&kind));
    }
    let mut switches: Vec<_> = world
        .query::<&SwitchPoint>()
        .iter(world)
        .map(|point| point.id)
        .collect();
    switches.sort_unstable();
    assert_eq!(switches, ["east_exit", "west_exit"]);

    let mut targets = world.query_filtered::<
        (&Sprite, &Collider, &Sensor, &CollidingEntities),
        With<Interactable>,
    >();
    assert_eq!(targets.iter(world).count(), 5);
    for (sprite, collider, _, _) in targets.iter(world) {
        let half_extents = collider.shape().as_cuboid().unwrap().half_extents;
        assert_eq!(
            Some(Vec2::new(half_extents.x, half_extents.y) * 2.0),
            sprite.custom_size,
            "the entire placeholder is the interaction area"
        );
    }

    // Road offsets must move both the sprites and their physical colliders.
    let offset = Vec3::new(1000.0, 200.0, 0.0);
    app.world_mut()
        .get_mut::<Transform>(root)
        .unwrap()
        .translation = offset;
    advance_physics(&mut app);
    let world = app.world_mut();
    for (local, global, position) in world
        .query_filtered::<
            (&Transform, &GlobalTransform, &Position),
            Or<(With<Platform>, With<Interactable>)>,
        >()
        .iter(world)
    {
        let expected = local.translation + offset;
        assert!(global.translation().abs_diff_eq(expected, 0.001));
        assert!(position.0.abs_diff_eq(expected.truncate(), 0.001));
    }
}

#[derive(Resource, Default)]
struct InteractionRequests(Vec<(Entity, Entity)>);

fn record_interaction(event: On<InteractRequested>, mut requests: ResMut<InteractionRequests>) {
    requests.0.push((event.entity, event.player));
}

fn record_requests(app: &mut App) {
    app.init_resource::<InteractionRequests>()
        .add_observer(record_interaction);
}

#[test]
fn e_requests_every_overlapping_target_once_with_the_requesting_player() {
    let mut app = headless_app();
    record_requests(&mut app);
    let root = road_entity(&mut app);
    let player = player_entity(&mut app);
    app.world_mut()
        .get_mut::<Transform>(root)
        .unwrap()
        .translation = Vec3::new(1000.0, 200.0, 0.0);
    let world = app.world_mut();
    let shop = world
        .query::<(Entity, &BuildingType)>()
        .iter(world)
        .find(|(_, kind)| **kind == BuildingType::Shop)
        .unwrap()
        .0;
    let east_exit = world
        .query::<(Entity, &SwitchPoint)>()
        .iter(world)
        .find(|(_, point)| point.id == "east_exit")
        .unwrap()
        .0;
    world.get_mut::<Transform>(east_exit).unwrap().translation = Vec3::new(20.0, -127.5, -1.0);

    // This is within the old radius, but neither collider overlaps the player.
    world.get_mut::<Transform>(player).unwrap().translation = Vec3::new(1060.0, 72.5, 0.0);
    advance_physics(&mut app);
    KeyCode::KeyE.press(app.world_mut());
    app.update();
    assert!(app.world().resource::<InteractionRequests>().0.is_empty());

    KeyCode::KeyE.release(app.world_mut());
    app.update();
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = Vec3::new(1000.0, 72.5, 0.0);
    advance_physics(&mut app);
    for target in [shop, east_exit] {
        assert!(
            app.world()
                .get::<CollidingEntities>(target)
                .unwrap()
                .contains(&player),
            "real physics must report the overlap before pressing E"
        );
    }
    assert!(app.world().resource::<InteractionRequests>().0.is_empty());

    KeyCode::KeyE.press(app.world_mut());
    app.update();
    let mut actual = app.world().resource::<InteractionRequests>().0.clone();
    actual.sort_unstable();
    let mut expected = vec![(shop, player), (east_exit, player)];
    expected.sort_unstable();
    assert_eq!(actual, expected);
    advance_physics(&mut app);
    assert_eq!(
        app.world().resource::<InteractionRequests>().0.len(),
        2,
        "holding E must not repeat requests"
    );

    KeyCode::KeyE.release(app.world_mut());
    app.update();
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = Vec3::new(0.0, 500.0, 0.0);
    advance_physics(&mut app);
    KeyCode::KeyE.press(app.world_mut());
    app.update();
    assert_eq!(app.world().resource::<InteractionRequests>().0.len(), 2);
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = Vec3::new(1000.0, 72.5, 0.0);
    advance_physics(&mut app);
    assert_eq!(
        app.world().resource::<InteractionRequests>().0.len(),
        2,
        "entering an overlap with E held must not send a request"
    );

    KeyCode::KeyE.release(app.world_mut());
    app.update();
    KeyCode::KeyE.press(app.world_mut());
    app.update();
    assert_eq!(app.world().resource::<InteractionRequests>().0.len(), 4);
}

#[test]
fn e_ignores_unmarked_colliders_and_collisions_with_non_players() {
    let mut app = headless_app();
    record_requests(&mut app);
    let root = road_entity(&mut app);
    app.world_mut().despawn(root);
    let player = player_entity(&mut app);
    let player_position = Vec3::new(600.0, 500.0, 0.0);
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = player_position;

    let spawn_sensor = |world: &mut World, position: Vec3| {
        world
            .spawn((
                Transform::from_translation(position),
                Collider::rectangle(80.0, 80.0),
                Sensor,
                CollidingEntities::default(),
            ))
            .id()
    };
    let unmarked = spawn_sensor(app.world_mut(), player_position);
    // Generic targets have no building or switch type; only Interactable matters.
    let target = spawn_sensor(app.world_mut(), player_position);
    app.world_mut().entity_mut(target).insert(Interactable);
    let other_position = Vec3::new(900.0, 500.0, 0.0);
    let other_target = spawn_sensor(app.world_mut(), other_position);
    app.world_mut().entity_mut(other_target).insert(Interactable);
    let non_player = app
        .world_mut()
        .spawn((
            Transform::from_translation(other_position),
            RigidBody::Kinematic,
            Collider::rectangle(30.0, 50.0),
        ))
        .id();
    advance_physics(&mut app);

    assert!(
        app.world()
            .get::<CollidingEntities>(unmarked)
            .unwrap()
            .contains(&player)
    );
    assert!(
        app.world()
            .get::<CollidingEntities>(target)
            .unwrap()
            .contains(&player)
    );
    let other_collisions = app.world().get::<CollidingEntities>(other_target).unwrap();
    assert!(other_collisions.contains(&non_player));
    assert!(!other_collisions.contains(&player));
    KeyCode::KeyE.press(app.world_mut());
    app.update();
    assert_eq!(
        app.world().resource::<InteractionRequests>().0,
        [(target, player)]
    );
}

#[test]
fn despawning_the_road_removes_all_its_parts_and_keeps_the_player() {
    let mut app = headless_app();
    let root = road_entity(&mut app);
    let children = app.world().get::<Children>(root).unwrap().to_vec();
    assert!(app.world_mut().despawn(root));
    assert!(app.world().get_entity(root).is_err());
    for child in children {
        assert!(app.world().get_entity(child).is_err());
    }
    let world = app.world_mut();
    assert_eq!(
        world
            .query_filtered::<Entity, With<Player>>()
            .iter(world)
            .count(),
        1
    );
    assert_eq!(world.query::<&Interactable>().iter(world).count(), 0);
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

impl LogOutput {
    fn interactions(&self) -> Vec<String> {
        String::from_utf8(self.0.lock().unwrap().clone())
            .unwrap()
            .lines()
            .filter(|line| {
                line.contains("Building interaction:") || line.contains("Switch point triggered:")
            })
            .map(str::to_owned)
            .collect()
    }
}

#[test]
fn every_building_and_switch_scene_binds_its_interaction_observer() {
    let output = LogOutput::default();
    let writer = output.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_target(false)
        .with_writer(move || writer.clone())
        .finish();

    tracing::subscriber::with_default(subscriber, || {
        let mut app = headless_app();
        advance_physics(&mut app);
        let player = player_entity(&mut app);
        let world = app.world_mut();
        let mut targets: Vec<_> = world
            .query::<(Entity, &BuildingType)>()
            .iter(world)
            .map(|(entity, kind)| (entity, format!("Building interaction: {kind:?}")))
            .collect();
        targets.extend(world
            .query::<(Entity, &SwitchPoint)>()
            .iter(world)
            .map(|(entity, point)| (entity, format!("Switch point triggered: {}", point.id))));
        assert_eq!(targets.len(), 5);

        for (index, (entity, expected)) in targets.into_iter().enumerate() {
            app.world_mut().trigger(InteractRequested { entity, player });
            let interactions = output.interactions();
            assert_eq!(interactions.len(), index + 1);
            assert!(interactions[index].contains(&expected));
        }
    });
}
