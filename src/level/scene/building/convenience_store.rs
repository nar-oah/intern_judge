use super::{BuildingDialogue, dialogue_building_scene};
use crate::config::{BuildingValues, ConvenienceKind};
use crate::game::{DailyTransactions, Money, Stamina};
use crate::level::RunningInteractRequested;
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
        #ConvenienceStore
        kind
        on(interact)
    }
}

fn interact(
    request: On<RunningInteractRequested>,
    stores: Query<&ConvenienceKind>,
    values: Res<BuildingValues>,
    money: Res<Money>,
    mut transactions: ResMut<DailyTransactions>,
    mut stamina: ResMut<Stamina>,
) {
    let kind = stores.get(request.entity).unwrap();
    let store = &values.convenience_stores[kind];
    if stamina.0 >= Stamina::MAX || money.0 + transactions.net() < store.price {
        return;
    }
    transactions.convenience_store_cost += store.price;
    stamina.0 = (stamina.0 + store.stamina_recovery).min(Stamina::MAX);
}
