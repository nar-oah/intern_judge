use super::building_scene;
use crate::game::{Money, Stamina};
use crate::level::{BuildingType, InteractRequested};
use bevy::prelude::*;

const COST: i32 = 10;
const RECOVERY: f32 = 30.0;

pub(in crate::level::scene) fn convenience_store_scene(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        @building_scene(position, size, Color::srgb(0.45, 0.7, 0.5))
        BuildingType::ConvenienceStore
        on(interact)
    }
}

fn interact(_: On<InteractRequested>, mut money: ResMut<Money>, mut stamina: ResMut<Stamina>) {
    if stamina.0 >= Stamina::MAX || money.0 < COST {
        return;
    }
    money.0 -= COST;
    stamina.0 = (stamina.0 + RECOVERY).min(Stamina::MAX);
}
