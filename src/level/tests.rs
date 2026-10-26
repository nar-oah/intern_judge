use super::*;
use crate::game::{HasHelmet, Money, Stamina};
use crate::physics::GameLayer;
use crate::player::{Player, PlayerAction, PlayerPlugin};
use crate::test_support::{advance, finish_app, headless_app};
use avian2d::prelude::*;
use bevy::platform::time::Instant;
use leafwing_input_manager::prelude::*;

#[derive(Resource, Default)]
struct Requests(Vec<(Entity, Entity)>);

fn record_requests(app: &mut App) {
    app.init_resource::<Requests>().add_observer(
        |event: On<InteractRequested>, mut requests: ResMut<Requests>| {
            requests.0.push((event.entity, event.player));
        },
    );
}

fn scene_app(gravity: Vec2) -> App {
    let mut app = headless_app(gravity);
    app.init_resource::<Money>()
        .init_resource::<Stamina>()
        .init_resource::<HasHelmet>()
        .add_plugins((PlayerPlugin, LevelPlugin));
    record_requests(&mut app);
    finish_app(&mut app);
    advance(&mut app, 2);
    app
}

fn interaction_app() -> App {
    let mut app = App::new();
    app.add_systems(PostUpdate, interaction::interact);
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

fn building(app: &mut App, kind: BuildingType) -> Entity {
    let world = app.world_mut();
    world
        .query::<(Entity, &BuildingType)>()
        .iter(world)
        .find(|(_, candidate)| **candidate == kind)
        .unwrap()
        .0
}

fn teleport(app: &mut App, entity: Entity, position: Vec3) {
    let local_position = app
        .world()
        .get::<ChildOf>(entity)
        .and_then(|parent| app.world().get::<GlobalTransform>(parent.parent()))
        .map(|parent| parent.affine().inverse().transform_point3(position))
        .unwrap_or(position);
    app.world_mut()
        .get_mut::<Transform>(entity)
        .unwrap()
        .translation = local_position;
    if let Some(mut physics_position) = app.world_mut().get_mut::<Position>(entity) {
        physics_position.0 = position.truncate();
    }
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
    let mut app = scene_app(Vec2::ZERO);
    let road = single::<RoadId>(&mut app);
    let player = single::<Player>(&mut app);
    let unrelated = app.world_mut().spawn_empty().id();
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
        if child == player {
            assert_eq!(world.get::<RigidBody>(child), Some(&RigidBody::Dynamic));
        } else if let Some(platform) = world.get::<Platform>(child) {
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
        BuildingType::ConvenienceStore,
        BuildingType::ToolStore,
        BuildingType::LotteryStore,
        BuildingType::Cat,
    ] {
        assert!(buildings.contains(&kind));
    }
    assert_eq!(switches, [&[scene::MARKET_ROAD, scene::COURT][..]]);

    let world = app.world_mut();
    assert!(world.despawn(road));
    assert!(world.get_entity(road).is_err());
    assert!(
        children
            .into_iter()
            .all(|child| world.get_entity(child).is_err())
    );
    assert!(world.get_entity(player).is_err());
    assert!(world.get_entity(unrelated).is_ok());
    assert_eq!(world.query::<&Interactable>().iter(world).count(), 0);
}

#[test]
fn interactions_request_an_overlapping_target_only_on_a_fresh_press() {
    let mut app = interaction_app();
    let player = player(&mut app);
    let target = target(&mut app, &[player]);

    interaction_frame(&mut app);
    assert!(requests(&app).is_empty(), "overlap alone does not interact");
    set_interact(&mut app, player, true);
    interaction_frame(&mut app);
    assert_eq!(requests(&app), [(target, player)]);
    interaction_frame(&mut app);
    assert_eq!(requests(&app).len(), 1, "holding interact does not repeat");

    app.world_mut()
        .get_mut::<CollidingEntities>(target)
        .unwrap()
        .clear();
    set_interact(&mut app, player, false);
    interaction_frame(&mut app);
    set_interact(&mut app, player, true);
    interaction_frame(&mut app);
    assert_eq!(
        requests(&app).len(),
        1,
        "a press without overlap does nothing"
    );
    app.world_mut()
        .get_mut::<CollidingEntities>(target)
        .unwrap()
        .insert(player);
    interaction_frame(&mut app);
    assert_eq!(requests(&app).len(), 1, "entering while held does nothing");
    set_interact(&mut app, player, false);
    interaction_frame(&mut app);
    set_interact(&mut app, player, true);
    interaction_frame(&mut app);
    assert_eq!(requests(&app), [(target, player), (target, player)]);
}

#[test]
fn interactions_ignore_unmarked_targets_and_non_players() {
    let mut app = interaction_app();
    let player = player(&mut app);
    let mut actions = ActionState::<PlayerAction>::default();
    actions.press(&PlayerAction::Interact);
    let non_player = app.world_mut().spawn(actions).id();
    target(&mut app, &[non_player]);
    app.world_mut()
        .spawn(CollidingEntities([player].into_iter().collect()));
    app.world_mut().spawn(Interactable);
    set_interact(&mut app, player, true);

    interaction_frame(&mut app);

    assert!(requests(&app).is_empty());
    app.world_mut().despawn(player);
    interaction_frame(&mut app);
    assert!(
        requests(&app).is_empty(),
        "no player during scene selection"
    );
}

#[test]
fn e_interacts_with_scene_sensors_only_during_physical_overlap() {
    let mut app = scene_app(Vec2::ZERO);
    let road = single::<RoadId>(&mut app);
    let player = single::<Player>(&mut app);
    teleport(&mut app, road, Vec3::new(1000.0, 200.0, 0.0));
    let shop = building(&mut app, BuildingType::ConvenienceStore);
    app.world_mut().resource_mut::<Stamina>().0 = 80.0;
    let world = app.world_mut();
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
    assert_eq!(app.world().resource::<Money>().0, 100);
    assert_eq!(app.world().resource::<Stamina>().0, 80.0);

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
    assert_eq!(app.world().resource::<Money>().0, 90);
    assert_eq!(app.world().resource::<Stamina>().0, Stamina::MAX);
    advance(&mut app, 3);
    assert_eq!(requests(&app).len(), 1, "holding E does not repeat");
    assert_eq!(app.world().resource::<Money>().0, 90);

    teleport(&mut app, player, outside);
    assert!(
        !app.world()
            .get::<CollidingEntities>(shop)
            .unwrap()
            .contains(&player)
    );
}

#[test]
fn e_uses_the_tool_and_lottery_scene_observers() {
    let mut app = scene_app(Vec2::ZERO);
    let player = single::<Player>(&mut app);
    let tool_store = building(&mut app, BuildingType::ToolStore);
    let center = app
        .world()
        .get::<GlobalTransform>(tool_store)
        .unwrap()
        .translation();
    teleport(&mut app, player, center);

    interact_key(&mut app, true);
    assert!(app.world().resource::<HasHelmet>().0);
    assert_eq!(app.world().resource::<Money>().0, 80);
    assert_eq!(requests(&app), [(tool_store, player)]);

    interact_key(&mut app, false);
    let lottery_store = building(&mut app, BuildingType::LotteryStore);
    let center = app
        .world()
        .get::<GlobalTransform>(lottery_store)
        .unwrap()
        .translation();
    teleport(&mut app, player, center);
    interact_key(&mut app, true);

    assert!([70, 80, 120].contains(&app.world().resource::<Money>().0));
    assert!(app.world().resource::<HasHelmet>().0);
    assert_eq!(
        requests(&app),
        [(tool_store, player), (lottery_store, player)]
    );
    let balance = app.world().resource::<Money>().0;
    advance(&mut app, 3);
    assert_eq!(app.world().resource::<Money>().0, balance);
}

#[test]
fn cat_requires_a_platform_jump_and_is_rescued_only_once() {
    let mut app = scene_app(Vec2::NEG_Y * 1000.0);
    let player = single::<Player>(&mut app);
    let cat = building(&mut app, BuildingType::Cat);
    teleport(&mut app, player, Vec3::new(170.0, -137.5, 0.0));
    advance(&mut app, 8);
    assert!(
        !app.world()
            .get::<CollidingEntities>(cat)
            .unwrap()
            .contains(&player)
    );
    interact_key(&mut app, true);
    assert!(requests(&app).is_empty());
    assert_eq!(app.world().resource::<Money>().0, 100);

    interact_key(&mut app, false);
    teleport(&mut app, player, Vec3::new(65.0, -137.5, 0.0));
    let ground_y = app.world().get::<Position>(player).unwrap().y;
    KeyCode::Space.press(app.world_mut());
    KeyCode::KeyD.press(app.world_mut());
    for _ in 0..96 {
        app.update();
        if app
            .world()
            .get::<CollidingEntities>(cat)
            .unwrap()
            .contains(&player)
        {
            break;
        }
    }
    KeyCode::Space.release(app.world_mut());
    KeyCode::KeyD.release(app.world_mut());
    assert!(app.world().get::<Position>(player).unwrap().y > ground_y + 50.0);
    assert!(
        app.world()
            .get::<CollidingEntities>(cat)
            .unwrap()
            .contains(&player),
        "the cat must be reachable by jumping onto its platform"
    );

    interact_key(&mut app, true);
    assert_eq!(app.world().resource::<Money>().0, 120);
    assert!(app.world().get_entity(cat).is_err());
    assert_eq!(requests(&app), [(cat, player)]);
    interact_key(&mut app, false);
    interact_key(&mut app, true);
    assert_eq!(app.world().resource::<Money>().0, 120);
    assert_eq!(requests(&app).len(), 1);
}
