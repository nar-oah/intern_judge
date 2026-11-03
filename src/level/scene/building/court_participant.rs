use super::building_scene;
use crate::{
    level::{InteractRequested, dialogue::ui::dialogue_box_scene},
    player::Player,
};
use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_replique::prelude::{DialogueRunner, StartDialogue};

#[derive(Component, Clone, Copy)]
pub(in crate::level::scene) enum CourtParticipant {
    Judge,
    Prosecution,
    Defense,
}

pub(in crate::level::scene) fn court_participant_scene(
    position: Vec2,
    participant: CourtParticipant,
) -> impl Scene {
    let (name, size, color) = match participant {
        CourtParticipant::Judge => (
            "JudgeBench",
            Vec2::new(120.0, 70.0),
            Color::srgb(0.65, 0.45, 0.2),
        ),
        CourtParticipant::Prosecution => (
            "Prosecution",
            Vec2::new(80.0, 70.0),
            Color::srgb(0.8, 0.3, 0.3),
        ),
        CourtParticipant::Defense => (
            "Defense",
            Vec2::new(80.0, 70.0),
            Color::srgb(0.3, 0.45, 0.8),
        ),
    };
    bsn! {
        Name::new(name)
        participant
        @building_scene(position, size, color)
        on(interact)
    }
}

fn interact(
    request: On<InteractRequested>,
    participants: Query<&CourtParticipant>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut players: Query<&mut LinearVelocity, (With<Player>, Without<DialogueRunner>)>,
    mut start: MessageWriter<StartDialogue>,
) {
    let Ok(mut velocity) = players.get_mut(request.player) else {
        return;
    };
    let node = match participants.get(request.entity).unwrap() {
        CourtParticipant::Judge => "verdict",
        CourtParticipant::Prosecution => "prosecution",
        CourtParticipant::Defense => "defense",
    };
    velocity.x = 0.0;
    commands
        .entity(request.player)
        .insert(DialogueRunner::new(assets.load("dialogue/court.rep")));
    commands.spawn_scene(dialogue_box_scene());
    start.write(StartDialogue {
        runner: request.player,
        node: node.into(),
    });
}
