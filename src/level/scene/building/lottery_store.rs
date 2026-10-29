use super::building_scene;
use crate::config::{BuildingValues, LotteryValues};
use crate::game::Money;
use crate::level::{BuildingType, InteractRequested};
use bevy::prelude::*;

pub(in crate::level::scene) fn lottery_store_scene(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        @building_scene(position, size, Color::srgb(0.65, 0.5, 0.75))
        BuildingType::LotteryStore
        on(interact)
    }
}

fn interact(_: On<InteractRequested>, values: Res<BuildingValues>, mut money: ResMut<Money>) {
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

#[cfg(test)]
mod tests;
