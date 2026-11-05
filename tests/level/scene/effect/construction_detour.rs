use super::*;
use crate::{
    level::{RoadEffect, RoadId, scene::road_scene},
    player::{Player, PlayerPlugin},
    test_support::{advance, finish_app, headless_app, single},
};
use avian2d::prelude::*;
use leafwing_input_manager::prelude::*;

#[test]
fn market_detour_blocks_walking_and_can_be_cleared_by_the_initial_jump() {
    let mut app = headless_app(Vec2::NEG_Y * 1000.0);
    app.add_plugins(PlayerPlugin);
    app.world_mut()
        .spawn_scene(road_scene(
            RoadId::MarketRoad,
            &[RoadEffect::ConstructionDetour],
            "",
            RoadId::MarketRoad,
        ))
        .unwrap();
    let player = single::<Player>(&mut app);
    let detour = single::<ConstructionDetour>(&mut app);
    let world = app.world_mut();
    let detour_x = world.get::<Transform>(detour).unwrap().translation.x;
    let mut position = world.get_mut::<Transform>(player).unwrap();
    position.translation.x = detour_x - 100.0;
    finish_app(&mut app);
    advance(&mut app, 30);

    KeyCode::KeyD.press(app.world_mut());
    advance(&mut app, 60);
    let blocked_x = app.world().get::<Position>(player).unwrap().x;
    assert!(blocked_x < detour_x);
    assert!(blocked_x > detour_x - 100.0);

    // Give the player a short run-up so their feet clear the obstacle before contact.
    KeyCode::KeyD.release(app.world_mut());
    KeyCode::KeyA.press(app.world_mut());
    advance(&mut app, 16);
    KeyCode::KeyA.release(app.world_mut());
    KeyCode::KeyD.press(app.world_mut());
    KeyCode::Space.press(app.world_mut());
    app.update();
    assert!(app.world().get::<LinearVelocity>(player).unwrap().y > 0.0);
    advance(&mut app, 70);
    assert!(app.world().get::<Position>(player).unwrap().x > detour_x + 60.0);
    assert!(app.world().get_entity(detour).is_ok());
}
