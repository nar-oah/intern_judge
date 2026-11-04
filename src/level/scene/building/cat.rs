use super::{BuildingDialogue, dialogue_building_scene};
use crate::config::BuildingValues;
use crate::game::{CommuteState, Money};
use crate::level::{BuildingType, InteractRequested};
use bevy::prelude::*;

pub(in crate::level::scene) fn cat_scene(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        @dialogue_building_scene(
            position,
            size,
            Color::srgb(0.95, 0.8, 0.55),
            BuildingDialogue::new("dialogue/buildings.rep", "cat").when_paused(),
        )
        BuildingType::Cat
        on(interact)
    }
}

fn interact(
    request: On<InteractRequested>,
    mut commands: Commands,
    mut money: ResMut<Money>,
    values: Res<BuildingValues>,
    commute: Res<State<CommuteState>>,
) {
    if *commute.get() != CommuteState::Running {
        return;
    }
    money.0 += values.cat_reward;
    commands.entity(request.entity).despawn();
}
