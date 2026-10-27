use super::building_scene;
use crate::game::{HasHelmet, Money};
use crate::level::{BuildingType, InteractRequested};
use bevy::prelude::*;

const COST: i32 = 20;

pub(in crate::level::scene) fn tool_store_scene(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        @building_scene(position, size, Color::srgb(0.9, 0.65, 0.3))
        BuildingType::ToolStore
        on(interact)
    }
}

fn interact(_: On<InteractRequested>, mut money: ResMut<Money>, mut helmet: ResMut<HasHelmet>) {
    if helmet.0 || money.0 < COST {
        return;
    }
    money.0 -= COST;
    helmet.0 = true;
}
