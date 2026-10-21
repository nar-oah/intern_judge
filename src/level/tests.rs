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

#[derive(Resource, Default)]
struct Requests(Vec<(Entity, Entity)>);

fn app() -> App {
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
    .init_resource::<Requests>()
    .add_observer(
        |event: On<InteractRequested>, mut requests: ResMut<Requests>| {
            requests.0.push((event.entity, event.player));
        },
    )
    .edit_schedule(PostUpdate, |schedule| {
        schedule.set_executor(SingleThreadedExecutor::new());
    });
    app.finish();
    app.cleanup();
    settle(&mut app);
    app
}

fn single<C: Component>(app: &mut App) -> Entity {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<C>>()
        .single(world)
        .unwrap()
}

fn settle(app: &mut App) {
    // Propagate transforms, advance fixed physics, and apply scene observers.
    for _ in 0..3 {
        app.update();
    }
}

fn teleport(app: &mut App, entity: Entity, position: Vec3) {
    app.world_mut()
        .get_mut::<Transform>(entity)
        .unwrap()
        .translation = position;
    settle(app);
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
fn road_scene_preserves_its_parts_sensor_dimensions_and_lifecycle() {
    let mut app = app();
    let road = single::<RoadId>(&mut app);
    let player = single::<Player>(&mut app);
    assert_eq!(
        app.world().get::<RoadId>(road),
        Some(&RoadId("example_road"))
    );
    let children = app.world().get::<Children>(road).unwrap().to_vec();
    assert_eq!(children.len(), 10);
    let offset = Vec3::new(1000.0, 200.0, 0.0);
    teleport(&mut app, road, offset);

    for &child in &children {
        let world = app.world();
        assert_eq!(world.get::<ChildOf>(child).unwrap().parent(), road);
        let sprite_size = world.get::<Sprite>(child).unwrap().custom_size.unwrap();
        let collider = world.get::<Collider>(child).unwrap();
        let half_extents = collider.shape().as_cuboid().unwrap().half_extents;
        assert_eq!(Vec2::new(half_extents.x, half_extents.y) * 2.0, sprite_size);
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
            assert_eq!(platform.size, sprite_size);
            assert_eq!(world.get::<RigidBody>(child), Some(&RigidBody::Static));
        } else {
            assert!(world.get::<Interactable>(child).is_some());
            assert!(world.get::<Sensor>(child).is_some());
            assert!(world.get::<CollidingEntities>(child).is_some());
        }
    }
    let world = app.world_mut();
    assert_eq!(world.query::<&Platform>().iter(world).count(), 5);
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
fn e_requests_all_physical_overlaps_only_on_a_new_press() {
    let mut app = app();
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
    let exit = world
        .query::<(Entity, &SwitchPoint)>()
        .iter(world)
        .find(|(_, point)| point.id == "east_exit")
        .unwrap()
        .0;
    teleport(&mut app, exit, Vec3::new(20.0, -127.5, -1.0));

    // Nearby centers are insufficient when the player's collider does not overlap.
    teleport(&mut app, player, Vec3::new(1060.0, 72.5, 0.0));
    interact_key(&mut app, true);
    assert!(requests(&app).is_empty());
    interact_key(&mut app, false);
    let overlap = Vec3::new(1000.0, 72.5, 0.0);
    teleport(&mut app, player, overlap);
    for target in [shop, exit] {
        assert!(
            app.world()
                .get::<CollidingEntities>(target)
                .unwrap()
                .contains(&player)
        );
    }
    assert!(requests(&app).is_empty(), "overlap alone does not interact");

    interact_key(&mut app, true);
    assert_eq!(requests(&app).len(), 2);
    for target in [shop, exit] {
        assert!(requests(&app).contains(&(target, player)));
    }
    settle(&mut app);
    assert_eq!(
        requests(&app).len(),
        2,
        "holding E does not repeat requests"
    );

    interact_key(&mut app, false);
    teleport(&mut app, player, Vec3::new(0.0, 500.0, 0.0));
    interact_key(&mut app, true);
    assert_eq!(requests(&app).len(), 2, "E without overlap does nothing");
    teleport(&mut app, player, overlap);
    assert_eq!(requests(&app).len(), 2, "entering with E held does nothing");
    interact_key(&mut app, false);
    interact_key(&mut app, true);
    assert_eq!(requests(&app).len(), 4, "a fresh E press interacts again");
}

fn sensor(app: &mut App, position: Vec3, marked: bool) -> Entity {
    let mut entity = app.world_mut().spawn((
        Transform::from_translation(position),
        Collider::rectangle(80.0, 80.0),
        Sensor,
        CollidingEntities::default(),
    ));
    if marked {
        entity.insert(Interactable);
    }
    entity.id()
}

#[test]
fn custom_targets_need_no_building_type_and_ignore_unrelated_collisions() {
    let mut app = app();
    let road = single::<RoadId>(&mut app);
    app.world_mut().despawn(road);
    let player = single::<Player>(&mut app);
    let position = Vec3::new(600.0, 500.0, 0.0);
    teleport(&mut app, player, position);
    let target = sensor(&mut app, position, true);
    let unmarked = sensor(&mut app, position, false);
    let other_position = Vec3::new(900.0, 500.0, 0.0);
    let other_target = sensor(&mut app, other_position, true);
    let non_player = app
        .world_mut()
        .spawn((
            Transform::from_translation(other_position),
            RigidBody::Kinematic,
            Collider::rectangle(30.0, 50.0),
        ))
        .id();
    settle(&mut app);

    for entity in [target, unmarked] {
        assert!(
            app.world()
                .get::<CollidingEntities>(entity)
                .unwrap()
                .contains(&player)
        );
    }
    let other_collisions = app.world().get::<CollidingEntities>(other_target).unwrap();
    assert!(other_collisions.contains(&non_player));
    assert!(!other_collisions.contains(&player));
    interact_key(&mut app, true);
    assert_eq!(requests(&app), [(target, player)]);
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
fn building_and_switch_scenes_bind_their_own_interaction_observers() {
    let output = LogOutput::default();
    let writer = output.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_target(false)
        .with_writer(move || writer.clone())
        .finish();
    tracing::subscriber::with_default(subscriber, || {
        let mut app = app();
        let player = single::<Player>(&mut app);
        let world = app.world_mut();
        let mut targets: Vec<_> = world
            .query::<(Entity, &BuildingType)>()
            .iter(world)
            .map(|(entity, kind)| (entity, format!("Building interaction: {kind:?}")))
            .collect();
        targets.extend(
            world
                .query::<(Entity, &SwitchPoint)>()
                .iter(world)
                .map(|(entity, point)| (entity, format!("Switch point triggered: {}", point.id))),
        );
        assert_eq!(targets.len(), 5);

        for (entity, _) in &targets {
            app.world_mut().trigger(InteractRequested {
                entity: *entity,
                player,
            });
        }
        let logs = String::from_utf8(output.0.lock().unwrap().clone()).unwrap();
        let interactions: Vec<_> = logs
            .lines()
            .filter(|line| {
                line.contains("Building interaction:") || line.contains("Switch point triggered:")
            })
            .collect();
        assert_eq!(interactions.len(), targets.len());
        for (_, expected) in targets {
            assert_eq!(
                interactions
                    .iter()
                    .filter(|line| line.contains(&expected))
                    .count(),
                1
            );
        }
    });
}
