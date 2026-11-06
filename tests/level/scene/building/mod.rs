use super::*;
use crate::{
    config::ConvenienceKind,
    game::{CommuteState, DailyTransactions, HasHelmet, Money, Stamina},
    level::{CurrentCase, RunningInteractRequested},
    test_support::{finish_app, level_app},
};
use bevy_replique::prelude::RepliquePlugin;

mod dialogue;
mod transactions;

fn building_app() -> App {
    let mut app = level_app();
    app.add_plugins(RepliquePlugin)
        .insert_resource(Money(100))
        .init_resource::<DailyTransactions>()
        .init_resource::<CurrentCase>()
        .init_resource::<Stamina>()
        .init_resource::<HasHelmet>();
    finish_app(&mut app);
    app
}

fn spawn_player(app: &mut App, velocity: Vec2) -> Entity {
    app.world_mut()
        .spawn((Player::default(), LinearVelocity(velocity)))
        .id()
}

fn interact(app: &mut App, entity: Entity) {
    let player = spawn_player(app, Vec2::new(42.0, 7.0));
    app.world_mut().trigger(RunningInteractRequested { entity });
    app.world_mut().flush();

    assert!(app.world().get::<DialogueRunner>(player).is_none());
    assert_eq!(
        app.world().get::<LinearVelocity>(player).unwrap().0,
        Vec2::new(42.0, 7.0)
    );
    assert!(app.world().resource::<Messages<StartDialogue>>().is_empty());
}

fn spawn_building(app: &mut App, building: &str) -> Entity {
    match building {
        "ConvenienceStore" => app
            .world_mut()
            .spawn_scene(convenience_store_scene(
                Vec2::ZERO,
                Vec2::ONE,
                ConvenienceKind::Small,
            ))
            .unwrap()
            .id(),
        "ToolStore" => app
            .world_mut()
            .spawn_scene(tool_store_scene(Vec2::ZERO, Vec2::ONE))
            .unwrap()
            .id(),
        "LotteryStore" => app
            .world_mut()
            .spawn_scene(lottery_store_scene(Vec2::ZERO, Vec2::ONE))
            .unwrap()
            .id(),
        "Cat" => app
            .world_mut()
            .spawn_scene(cat_scene(Vec2::ZERO, Vec2::ONE))
            .unwrap()
            .id(),
        _ => panic!("unknown building: {building}"),
    }
}
