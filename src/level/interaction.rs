use super::{BuildingType, Interactable, SwitchPoint};
use crate::player::{Player, PlayerAction};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub(super) fn interact(
    players: Query<(&ActionState<PlayerAction>, &GlobalTransform), With<Player>>,
    targets: Query<(Entity, &GlobalTransform, &Interactable)>,
    buildings: Query<&BuildingType>,
    switch_points: Query<&SwitchPoint>,
) {
    for (actions, transform) in &players {
        if !actions.just_pressed(&PlayerAction::Interact) {
            continue;
        }

        let player_position = transform.translation().truncate();
        // One press activates only the closest target, even if ranges overlap.
        let nearest = targets
            .iter()
            .filter_map(|(entity, transform, interactable)| {
                let distance = player_position.distance_squared(transform.translation().truncate());
                (distance <= interactable.radius * interactable.radius)
                    .then_some((entity, distance))
            })
            .min_by(|(_, a), (_, b)| a.total_cmp(b));

        let Some((entity, _)) = nearest else {
            continue;
        };

        if let Ok(building_type) = buildings.get(entity) {
            info!("Building interaction: {building_type:?}");
        } else if let Ok(point) = switch_points.get(entity) {
            info!("Switch point triggered: {}", point.id);
        }
    }
}
