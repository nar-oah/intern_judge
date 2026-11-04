#[cfg(test)]
mod tests;
use super::{BuildingDialogue, dialogue_building_scene};
use crate::config::{BuildingValues, LotteryValues};
use crate::game::{CommuteState, Money};
use crate::level::{BuildingType, InteractRequested};
use bevy::prelude::*;

pub(in crate::level::scene) fn lottery_store_scene(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        @dialogue_building_scene(
            position,
            size,
            Color::srgb(0.65, 0.5, 0.75),
            BuildingDialogue::new("dialogue/buildings.rep", "lottery_store").when_paused(),
        )
        BuildingType::LotteryStore
        on(interact)
    }
}

fn interact(
    _: On<InteractRequested>,
    values: Res<BuildingValues>,
    mut money: ResMut<Money>,
    commute: Res<State<CommuteState>>,
) {
    if *commute.get() != CommuteState::Running {
        return;
    }
    let lottery = &values.lottery;
    if money.0 < lottery.price {
        return;
    }
    let total_weight = lottery.prizes.iter().map(|prize| prize.weight).sum();
    money.0 -= lottery.price;
    money.0 += prize(lottery, rand::random_range(0..total_weight));
}

fn prize(lottery: &LotteryValues, mut roll: u32) -> i32 {
    for prize in &lottery.prizes {
        if roll < prize.weight {
            return prize.reward;
        }
        roll -= prize.weight;
    }
    unreachable!("lottery roll must be within configured prize weights")
}
