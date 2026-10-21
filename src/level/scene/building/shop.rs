use super::building_scene;
use crate::level::{BuildingType, InteractRequested};
use bevy::prelude::*;

pub(in crate::level::scene) fn shop_scene(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        @building_scene(position, size, Color::srgb(0.9, 0.65, 0.3))
        BuildingType::Shop
        on(interact)
    }
}

fn interact(request: On<InteractRequested>) {
    info!(player = ?request.player, "Building interaction: Shop");
}
