use super::{BuildingDialogue, dialogue_building_scene};
use crate::config::{BuildingValues, ConvenienceKind};
use crate::game::{Money, Stamina};
use crate::level::{BuildingType, RunningInteractRequested};
use bevy::prelude::*;

pub(in crate::level::scene) fn convenience_store_scene(
    position: Vec2,
    size: Vec2,
    kind: ConvenienceKind,
) -> impl Scene {
    bsn! {
        @dialogue_building_scene(
            position,
            size,
            Color::srgb(0.45, 0.7, 0.5),
            BuildingDialogue::new("dialogue/buildings.rep", "convenience_store"),
        )
        BuildingType::ConvenienceStore
        kind
        on(interact)
    }
}

fn interact(
    request: On<RunningInteractRequested>,
    stores: Query<&ConvenienceKind>,
    values: Res<BuildingValues>,
    mut money: ResMut<Money>,
    mut stamina: ResMut<Stamina>,
) {
    let kind = stores.get(request.entity).unwrap();
    let store = &values.convenience_stores[kind];
    if stamina.0 >= Stamina::MAX || money.0 < store.price {
        return;
    }
    money.0 -= store.price;
    stamina.0 = (stamina.0 + store.stamina_recovery).min(Stamina::MAX);
}
