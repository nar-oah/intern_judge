use super::building_scene;
use crate::{
    level::{InteractRequested, dialogue::ui::dialogue_box_scene},
    player::Player,
};
use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_replique::prelude::{DialogueRunner, StartDialogue};

pub(in crate::level::scene) fn judge_bench_scene(position: Vec2) -> impl Scene {
    bsn! {
        #JudgeBench
        @building_scene(
            position,
            Vec2::new(120.0, 70.0),
            Color::srgb(0.65, 0.45, 0.2),
        )
        on(interact)
    }
}

fn interact(
    request: On<InteractRequested>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut players: Query<&mut LinearVelocity, (With<Player>, Without<DialogueRunner>)>,
    mut start: MessageWriter<StartDialogue>,
) {
    let Ok(mut velocity) = players.get_mut(request.player) else {
        return;
    };
    velocity.x = 0.0;
    commands
        .entity(request.player)
        .insert(DialogueRunner::new(assets.load("dialogue/court.rep")));
    commands.spawn_scene(dialogue_box_scene());
    start.write(StartDialogue {
        runner: request.player,
        node: "verdict".into(),
    });
}
