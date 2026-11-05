use super::{Interactable, PausedInteractRequested, RunningInteractRequested};
use crate::{
    game::CommuteState,
    player::{Player, PlayerAction, Stunned},
};
use avian2d::prelude::CollidingEntities;
use bevy::prelude::*;
use bevy_replique::prelude::DialogueRunner;
use leafwing_input_manager::prelude::*;

type InteractablePlayerFilter = (With<Player>, Without<Stunned>, Without<DialogueRunner>);

pub(super) fn interact(
    mut commands: Commands,
    commute: Res<State<CommuteState>>,
    players: Query<(Entity, &ActionState<PlayerAction>), InteractablePlayerFilter>,
    targets: Query<(Entity, &CollidingEntities), With<Interactable>>,
) {
    let Ok((player, actions)) = players.single() else {
        return;
    };
    if !actions.just_pressed(&PlayerAction::Interact) {
        return;
    }
    for (entity, collisions) in &targets {
        if collisions.contains(&player) {
            match commute.get() {
                CommuteState::Running => {
                    commands.trigger(RunningInteractRequested { entity, player });
                }
                CommuteState::Paused => {
                    commands.trigger(PausedInteractRequested { entity, player });
                }
            }
            break;
        }
    }
}
