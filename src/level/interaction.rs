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
    for (player, actions) in &players {
        if !actions.just_pressed(&PlayerAction::Interact) {
            continue;
        }

        for (entity, collisions) in &targets {
            if collisions.contains(&player) {
                commands.trigger(InteractRequested { entity, player });
            }
        }
    }
}
