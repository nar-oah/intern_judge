use super::building_scene;
use crate::level::{BuildingType, InteractRequested};
use bevy::prelude::*;

pub(in crate::level::scene) fn factory_scene(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        @building_scene(position, size, Color::srgb(0.65, 0.5, 0.75))
        BuildingType::Factory
        on(interact)
    }
}

fn interact(request: On<InteractRequested>) {
    info!(player = ?request.player, "Building interaction: Factory");
}
