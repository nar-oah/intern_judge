use super::{BuildingDialogue, dialogue_building_scene};
use crate::config::BuildingValues;
use crate::game::DailyTransactions;
use crate::level::RunningInteractRequested;
use bevy::prelude::*;

pub(in crate::level::scene) fn cat_scene(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        @dialogue_building_scene(
            position,
            size,
            Color::srgb(0.95, 0.8, 0.55),
            BuildingDialogue::new("dialogue/buildings.rep", "cat"),
        )
        #Cat
        on(interact)
    }
}

fn interact(
    request: On<RunningInteractRequested>,
    mut commands: Commands,
    mut transactions: ResMut<DailyTransactions>,
    values: Res<BuildingValues>,
) {
    transactions.cat_reward += values.cat_reward;
    commands.entity(request.entity).despawn();
}
