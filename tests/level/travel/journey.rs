use crate::{
    config::{BuildingValues, CaseDefs, ConvenienceKind, GameSettings},
    game::{CommuteState, CommuteTime, HasHelmet, Money, Stamina},
    level::{
        CurrentCase, RoadEffect, RoadEffects, RoadId, SceneRoot, SwitchPoint, TodayRoads,
        tests::support::{
            activate, assert_commute, assert_map, assert_road, assert_today_roads, weekday_app,
        },
    },
    player::Player,
    test_support::{advance, assert_removed, press_e, scene_entities, single},
};
use avian2d::prelude::{CollidingEntities, LinearVelocity, Position};
use bevy::prelude::*;

fn teleport(app: &mut App, entity: Entity, position: Vec3) {
    let local = app
        .world()
        .get::<ChildOf>(entity)
        .and_then(|parent| app.world().get::<GlobalTransform>(parent.parent()))
        .map_or(position, |parent| {
            parent.affine().inverse().transform_point3(position)
        });
    app.world_mut()
        .get_mut::<Transform>(entity)
        .unwrap()
        .translation = local;
    app.world_mut().get_mut::<Position>(entity).unwrap().0 = position.truncate();
    app.world_mut().get_mut::<LinearVelocity>(entity).unwrap().0 = Vec2::ZERO;
    advance(app, 3);
}

#[test]
fn commute_journey_replaces_scenes_and_preserves_global_progress() {
    let mut app = weekday_app(28);
    let camera = app.world_mut().spawn(Camera2d).id();
    let unrelated = app.world_mut().spawn_empty().id();

    let (_, player) = assert_road(&mut app, RoadId::ExampleRoad);
    assert_today_roads(&app, &[RoadId::ExampleRoad]);
    assert!(app.world().resource::<CurrentCase>().0.is_none());
    let spawn = app.world().get::<Position>(player).unwrap().0;
    assert_commute(&app, CommuteState::Running);
    let initial_elapsed = app.world().resource::<CommuteTime>().elapsed;
    advance(&mut app, 3);
    assert!(app.world().resource::<CommuteTime>().elapsed > initial_elapsed);

    // A real shop sensor connects keyboard input, collision detection, and its observer.
    let shop = {
        let world = app.world_mut();
        world
            .query::<(Entity, &Name)>()
            .iter(world)
            .find_map(|(entity, name)| (name.as_str() == "ConvenienceStore").then_some(entity))
            .unwrap()
    };
    let price =
        app.world().resource::<BuildingValues>().convenience_stores[&ConvenienceKind::Small].price;
    let balance = price * 3 + 1;
    app.world_mut().resource_mut::<Money>().0 = balance;
    app.world_mut().resource_mut::<Stamina>().0 = 1.0;
    press_e(&mut app);
    assert_eq!(app.world().resource::<Money>().0, balance);
    let center = app
        .world()
        .get::<GlobalTransform>(shop)
        .unwrap()
        .translation();
    teleport(&mut app, player, center);
    assert!(
        app.world()
            .get::<CollidingEntities>(shop)
            .unwrap()
            .contains(&player)
    );
    assert_eq!(
        app.world().resource::<Money>().0,
        balance,
        "entering while E is held does nothing"
    );
    press_e(&mut app);
    let after_purchase = app.world().resource::<Money>().0;
    assert!(after_purchase < balance);
    assert!(app.world().resource::<Stamina>().0 > 1.0);
    advance(&mut app, 3);
    assert_eq!(
        app.world().resource::<Money>().0,
        after_purchase,
        "holding E does not purchase again"
    );

    app.world_mut().resource_mut::<Money>().0 = 37;
    app.world_mut().resource_mut::<Stamina>().0 = 42.0;
    app.world_mut().resource_mut::<HasHelmet>().0 = true;
    app.world_mut().resource_mut::<GameSettings>().commute_limit = 123.0;
    // This road's existing effect makes its case unavailable when court is entered.
    app.world_mut()
        .resource_mut::<RoadEffects>()
        .0
        .insert(RoadId::ExampleRoad, vec![RoadEffect::FallingObject]);
    for (destination, choices) in [
        (RoadId::MarketRoad, &[RoadId::MarketRoad, RoadId::Court][..]),
        (RoadId::Court, &[RoadId::Court][..]),
    ] {
        let road = single::<SceneRoot>(&mut app);
        let player = single::<Player>(&mut app);
        let old_road = scene_entities(app.world(), road);
        let before = app.world().resource::<CommuteTime>().elapsed;
        advance(&mut app, 3);
        assert_commute(&app, CommuteState::Running);
        assert!(app.world().resource::<CommuteTime>().elapsed > before);

        let point = single::<SwitchPoint>(&mut app);
        let visited = app.world().resource::<TodayRoads>().0.clone();
        let center = app
            .world()
            .get::<GlobalTransform>(point)
            .unwrap()
            .translation();
        teleport(&mut app, player, center);
        assert!(
            app.world()
                .get::<CollidingEntities>(point)
                .unwrap()
                .contains(&player)
        );
        press_e(&mut app);
        app.update();
        assert_removed(&app, &old_road);
        let map = assert_map(&mut app, choices);
        assert_eq!(app.world().resource::<TodayRoads>().0, visited);
        let old_map = scene_entities(app.world(), map);
        assert_commute(&app, CommuteState::Paused);
        let paused_at = app.world().resource::<CommuteTime>().elapsed;
        advance(&mut app, 4);
        assert_eq!(app.world().resource::<CommuteTime>().elapsed, paused_at);

        activate(&mut app, destination);
        assert_removed(&app, &old_map);
        let (_, next_player) = assert_road(&mut app, destination);
        assert_today_roads(&app, &[RoadId::ExampleRoad, RoadId::MarketRoad]);
        if destination == RoadId::Court {
            let selected = app.world().resource::<CurrentCase>().0.as_ref().unwrap();
            assert_eq!(selected.road, RoadId::MarketRoad);
            assert!(
                app.world()
                    .resource::<TodayRoads>()
                    .0
                    .contains(&selected.road)
            );
            assert_eq!(selected.case, app.world().resource::<CaseDefs>().0[0]);
        } else {
            assert!(app.world().resource::<CurrentCase>().0.is_none());
        }
        assert_ne!(next_player, player);
        assert_eq!(app.world().get::<Position>(next_player).unwrap().0, spawn);
        assert_eq!(
            app.world().get::<LinearVelocity>(next_player).unwrap().0,
            Vec2::ZERO
        );
        advance(&mut app, 3);
        if destination == RoadId::Court {
            assert_commute(&app, CommuteState::Paused);
            assert_eq!(app.world().resource::<CommuteTime>().elapsed, paused_at);
        } else {
            assert_commute(&app, CommuteState::Running);
            assert!(app.world().resource::<CommuteTime>().elapsed > paused_at);
        }
        assert!(app.world().get_entity(camera).is_ok());
        assert!(app.world().get_entity(unrelated).is_ok());
        assert_eq!(app.world().resource::<Money>().0, 37);
        assert_eq!(app.world().resource::<Stamina>().0, 42.0);
        assert!(app.world().resource::<HasHelmet>().0);
        assert_eq!(app.world().resource::<GameSettings>().commute_limit, 123.0);
    }
}
