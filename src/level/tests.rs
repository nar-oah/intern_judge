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

#[test]
fn road_scene_groups_all_parts_and_preserves_platform_dimensions() {
    let mut app = headless_app();
    let root = road_entity(&mut app);
    assert_eq!(app.world().get::<RoadId>(root), Some(&RoadId("example_road")));
    let children: Vec<_> = app.world().get::<Children>(root).unwrap().iter().collect();
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
    let buildings: Vec<_> = world.query::<&BuildingType>().iter(world).copied().collect();
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

    // Road offsets must move both the sprites and their physical colliders.
    let offset = Vec3::new(1000.0, 200.0, 0.0);
    app.world_mut().get_mut::<Transform>(root).unwrap().translation = offset;
    app.update();
    let world = app.world_mut();
    for (local, global, position) in world
        .query_filtered::<(&Transform, &GlobalTransform, &Position), With<Platform>>()
        .iter(world)
    {
        let expected = local.translation + offset;
        assert!(global.translation().abs_diff_eq(expected, 0.001));
        assert!(position.0.abs_diff_eq(expected.truncate(), 0.001));
    }
}

#[test]
fn despawning_the_road_removes_all_its_parts_and_keeps_the_player() {
    let mut app = headless_app();
    let root = road_entity(&mut app);
    let children: Vec<_> = app.world().get::<Children>(root).unwrap().iter().collect();
    assert!(app.world_mut().despawn(root));
    assert!(app.world().get_entity(root).is_err());
    for child in children {
        assert!(app.world().get_entity(child).is_err());
    }
    let world = app.world_mut();
    assert_eq!(world.query_filtered::<Entity, With<Player>>().iter(world).count(), 1);
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
fn e_interacts_once_with_the_nearest_target_in_world_space() {
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
        let root = road_entity(&mut app);
        app.world_mut().get_mut::<Transform>(root).unwrap().translation =
            Vec3::new(1000.0, 200.0, 0.0);
        let world = app.world_mut();
        let player = world
            .query_filtered::<Entity, With<Player>>()
            .single(world)
            .unwrap();
        let east_exit = world
            .query::<(Entity, &SwitchPoint)>()
            .iter(world)
            .find(|(_, point)| point.id == "east_exit")
            .unwrap()
            .0;
        // Deliberately overlap the shop's range and the switch's range.
        world.get_mut::<Transform>(east_exit).unwrap().translation =
            Vec3::new(20.0, -127.5, -1.0);
        world.get_mut::<Transform>(player).unwrap().translation =
            Vec3::new(1000.0, 72.5, 0.0);
        app.update();
        assert!(output.interactions().is_empty(), "proximity alone is insufficient");

        KeyCode::KeyE.press(app.world_mut());
        app.update();
        let interactions = output.interactions();
        assert_eq!(interactions.len(), 1);
        assert!(interactions[0].contains("Building interaction: Shop"));
        app.update();
        assert_eq!(output.interactions().len(), 1, "held E must not retrigger");

        KeyCode::KeyE.release(app.world_mut());
        app.update();
        app.world_mut().get_mut::<Transform>(player).unwrap().translation =
            Vec3::new(1020.0, 72.5, 0.0);
        KeyCode::KeyE.press(app.world_mut());
        app.update();
        let interactions = output.interactions();
        assert_eq!(interactions.len(), 2);
        assert!(interactions[1].contains("Switch point triggered: east_exit"));

        KeyCode::KeyE.release(app.world_mut());
        app.update();
        app.world_mut().get_mut::<Transform>(player).unwrap().translation =
            Vec3::new(0.0, 500.0, 0.0);
        KeyCode::KeyE.press(app.world_mut());
        app.update();
        assert_eq!(output.interactions().len(), 2, "distant E must do nothing");
        app.world_mut().get_mut::<Transform>(player).unwrap().translation =
            Vec3::new(1000.0, 72.5, 0.0);
        app.update();
        assert_eq!(output.interactions().len(), 2, "entering range with E held must do nothing");
    });
}
