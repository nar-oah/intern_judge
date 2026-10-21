use super::{InteractRequested, Interactable};
use crate::player::{Player, PlayerAction};
use avian2d::prelude::CollidingEntities;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub(super) fn interact(
    mut commands: Commands,
    players: Query<(Entity, &ActionState<PlayerAction>), With<Player>>,
    targets: Query<(Entity, &CollidingEntities), With<Interactable>>,
) {
    let is_interact =
        |actions: &&ActionState<PlayerAction>| actions.just_pressed(&PlayerAction::Interact);
    for (player, _) in players.iter().filter(|(_, actions)| is_interact(actions)) {
        for (entity, _) in targets.iter().filter(|(_, col)| col.contains(&player)) {
            commands.trigger(InteractRequested { entity, player });
        }
    }
}
