use super::building_scene;
use crate::level::{BuildingType, InteractRequested};
use bevy::prelude::*;

pub(in crate::level::scene) fn residential_scene(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        @building_scene(position, size, Color::srgb(0.45, 0.7, 0.5))
        BuildingType::Residential
        on(interact)
    }
}

fn interact(request: On<InteractRequested>) {
    info!(player = ?request.player, "Building interaction: Residential");
}
