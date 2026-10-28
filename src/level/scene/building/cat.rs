use super::building_scene;
use crate::config::BuildingValues;
use crate::game::Money;
use crate::level::{BuildingType, InteractRequested};
use bevy::prelude::*;

pub(in crate::level::scene) fn cat_scene(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        @building_scene(position, size, Color::srgb(0.95, 0.8, 0.55))
        BuildingType::Cat
        on(interact)
    }
}

fn interact(
    request: On<InteractRequested>,
    mut commands: Commands,
    mut money: ResMut<Money>,
    values: Res<BuildingValues>,
) {
    money.0 += values.cat_reward;
    commands.entity(request.entity).despawn();
}
