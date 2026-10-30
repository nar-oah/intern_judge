use super::{InteractRequested, Interactable};
use crate::player::{Player, PlayerAction, Stunned};
use avian2d::prelude::CollidingEntities;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub(super) fn interact(
    mut commands: Commands,
    players: Query<(Entity, &ActionState<PlayerAction>), (With<Player>, Without<Stunned>)>,
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
            commands.trigger(InteractRequested { entity, player });
            break;
        }
    }
}
