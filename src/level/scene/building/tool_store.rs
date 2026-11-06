use super::{BuildingDialogue, dialogue_building_scene};
use crate::game::{DailyTransactions, HasHelmet, Money};
use crate::level::RunningInteractRequested;
use bevy::prelude::*;

const COST: i32 = 20;

pub(in crate::level::scene) fn tool_store_scene(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        @dialogue_building_scene(
            position,
            size,
            Color::srgb(0.9, 0.65, 0.3),
            BuildingDialogue::new("dialogue/buildings.rep", "tool_store"),
        )
        #ToolStore
        on(interact)
    }
}

fn interact(
    _: On<RunningInteractRequested>,
    money: Res<Money>,
    mut transactions: ResMut<DailyTransactions>,
    mut helmet: ResMut<HasHelmet>,
) {
    if helmet.0 || money.0 + transactions.net() < COST {
        return;
    }
    transactions.tool_store_cost += COST;
    helmet.0 = true;
}
